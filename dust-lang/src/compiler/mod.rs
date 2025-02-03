//! The Dust compiler and its accessories.
//!
//! This module provides two compilation options:
//! - [`compile`] is a simple function that borrows a string and returns a chunk, handling
//!   compilation and turning any resulting error into a [`DustError`], which can easily display a
//!   detailed report. The main chunk will be named "main".
//! - [`Compiler`] is created with a [`Lexer`] and protentially emits a [`CompileError`] or
//!   [`LexError`] if the input is invalid. Allows passing a name for the main chunk when
//!   [`Compiler::finish`] is called.
//!
//! # Errors
//!
//! The compiler can return errors due to:
//!     - Lexing errors
//!     - Parsing errors
//!     - Type conflicts
//!
//! It is a logic error to call [`Compiler::finish`] on a compiler that has emitted an error and
//! pass that chunk to the VM. Otherwise, if the compiler gives no errors and the VM encounters a
//! runtime error, it is the compiler's fault and the error should be fixed here.
mod error;
mod optimize;
mod parse_rule;
mod type_checks;

pub use error::CompileError;
use parse_rule::{ParseRule, Precedence};
use tracing::{Level, debug, info, span};
use type_checks::{check_math_type, check_math_types};

use std::{mem::replace, sync::Arc};

use optimize::control_flow_register_consolidation;

use crate::{
    Chunk, ConcreteValue, DustError, DustString, FunctionType, Instruction, Lexer, Local,
    NativeFunction, Operand, Operation, Scope, Span, Token, TokenKind, Type, Value,
    instruction::{CallNative, Close, Jump, LoadList, Point, Return, TypeCode},
};

/// Compiles the input and returns a chunk.
///
/// # Example
///
/// ```
/// # use dust_lang::compile;
/// let source = "40 + 2 == 42";
/// let chunk = compile(source).unwrap();
///
/// assert_eq!(chunk.instructions().len(), 3);
/// ```
pub fn compile(source: &str) -> Result<Chunk, DustError> {
    let lexer = Lexer::new(source);
    let mut compiler =
        Compiler::new(lexer, None, true).map_err(|error| DustError::compile(error, source))?;

    compiler
        .compile()
        .map_err(|error| DustError::compile(error, source))?;

    let chunk = compiler.finish();

    Ok(chunk)
}

/// The Dust compiler assembles a [`Chunk`] for the Dust VM. Any unrecognized symbols, disallowed
/// syntax or conflicting type usage will result in an error.
///
/// See the [`compile`] function an example of how to create and use a Compiler.
#[derive(Debug)]
pub struct Compiler<'src> {
    /// Used to get tokens for the compiler.
    lexer: Lexer<'src>,

    /// Name of the function being compiled. This is used to identify recursive calls, so it should
    /// be `None` for the main chunk. The main chunk can still be named by passing a name to
    /// [`Compiler::finish`], which will override this value.
    function_name: Option<DustString>,

    /// Type of the function being compiled. This is assigned to the chunk when [`Compiler::finish`]
    /// is called.
    r#type: FunctionType,

    /// Instructions, along with their types and positions, that have been compiled. The
    /// instructions and positions are assigned to the chunk when [`Compiler::finish`] is called.
    /// The types are discarded after compilation.
    instructions: Vec<(Instruction, Type, Span)>,

    /// Constants that have been compiled. These are assigned to the chunk when [`Compiler::finish`]
    /// is called.
    constants: Vec<Value>,

    /// Block-local variables and their types. The locals are assigned to the chunk when
    /// [`Compiler::finish`] is called. The types are discarded after compilation.
    locals: Vec<(Local, Type)>,

    /// Prototypes that have been compiled. These are assigned to the chunk when
    /// [`Compiler::finish`] is called.
    prototypes: Vec<Arc<Chunk>>,

    /// Maximum stack size required by the chunk. This is assigned to the chunk when
    /// [`Compiler::finish`] is called.
    stack_size: usize,

    /// The first register index that the compiler should use. This is used to avoid reusing the
    /// registers that are used for the function's arguments.
    minimum_register: u16,

    /// Index of the current block. This is used to determine the scope of locals and is incremented
    /// when a new block is entered.
    block_index: u8,

    /// The current block scope of the compiler. This is used to test if a variable is in scope.
    current_scope: Scope,

    /// Index of the Chunk in its parent's prototype list. This is set to 0 for the main chunk but
    /// that value is never read because the main chunk is not a callable function.
    prototype_index: u16,

    /// Whether the chunk is the program's main chunk. This is used to prevent recursive calls to
    /// the main chunk.
    is_main: bool,

    current_token: Token<'src>,
    current_position: Span,
    previous_token: Token<'src>,
    previous_position: Span,
}

impl<'src> Compiler<'src> {
    /// Creates a new compiler.
    pub fn new(
        mut lexer: Lexer<'src>,
        function_name: Option<DustString>,
        is_main: bool,
    ) -> Result<Self, CompileError> {
        let (current_token, current_position) = lexer.next_token()?;

        Ok(Compiler {
            function_name,
            r#type: FunctionType {
                type_parameters: Vec::with_capacity(0),
                value_parameters: Vec::with_capacity(0),
                return_type: Type::None,
            },
            instructions: Vec::new(),
            constants: Vec::new(),
            locals: Vec::new(),
            prototypes: Vec::new(),
            stack_size: 0,
            lexer,
            minimum_register: 0,
            block_index: 0,
            current_scope: Scope::default(),
            prototype_index: 0,
            is_main,
            current_token,
            current_position,
            previous_token: Token::Eof,
            previous_position: Span(0, 0),
        })
    }

    /// Compiles the source (which is in the lexer) while checking for errors and returning a
    /// [`CompileError`] if any are found. After calling this function, check its return value for
    /// an error, then call [`Compiler::finish`] to get the compiled chunk.
    pub fn compile(&mut self) -> Result<(), CompileError> {
        let span = span!(Level::INFO, "Compile");
        let _enter = span.enter();

        info!(
            "Begin chunk with {} at {}",
            self.current_token.to_string(),
            self.current_position.to_string()
        );

        loop {
            self.parse(Precedence::None)?;

            if matches!(self.current_token, Token::Eof | Token::RightBrace) {
                if self.get_last_operation() == Some(Operation::RETURN) {
                    break;
                }

                self.parse_implicit_return()?;

                break;
            }
        }

        info!("End chunk");

        Ok(())
    }

    /// Creates a new chunk with the compiled data, optionally assigning a name to the chunk.
    ///
    /// Note for maintainers: Do not give a name when compiling functions, only the main chunk. This
    /// will allow [`Compiler::function_name`] to be both the name used for recursive calls and the
    /// name of the function when it is compiled. The name can later be seen in the VM's call stack.
    pub fn finish(self) -> Chunk {
        let (instructions, positions): (Vec<Instruction>, Vec<Span>) = self
            .instructions
            .into_iter()
            .map(|(instruction, _, position)| (instruction, position))
            .unzip();
        let locals = self
            .locals
            .into_iter()
            .map(|(local, _)| local)
            .collect::<Vec<Local>>();

        Chunk {
            name: self.function_name,
            r#type: self.r#type,
            instructions,
            positions,
            constants: self.constants.to_vec(),
            locals,
            prototypes: self.prototypes,
            register_count: self.stack_size,
            prototype_index: self.prototype_index,
        }
    }

    fn is_eof(&self) -> bool {
        matches!(self.current_token, Token::Eof)
    }

    fn next_register(&self) -> u16 {
        self.instructions
            .iter()
            .rev()
            .find_map(|(instruction, _, _)| {
                if instruction.yields_value() {
                    Some(instruction.a_field() + 1)
                } else {
                    None
                }
            })
            .unwrap_or(self.minimum_register)
    }

    fn advance(&mut self) -> Result<(), CompileError> {
        if self.is_eof() {
            return Ok(());
        }

        let (new_token, position) = self.lexer.next_token()?;

        info!(
            "Parsing {} at {}",
            new_token.to_string(),
            position.to_string()
        );

        self.previous_token = replace(&mut self.current_token, new_token);
        self.previous_position = replace(&mut self.current_position, position);

        Ok(())
    }

    fn get_local(&self, index: u16) -> Result<&(Local, Type), CompileError> {
        self.locals
            .get(index as usize)
            .ok_or(CompileError::UndeclaredVariable {
                identifier: format!("#{}", index),
                position: self.current_position,
            })
    }

    fn get_local_index(&self, identifier_text: &str) -> Result<u16, CompileError> {
        self.locals
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, (local, _))| {
                let constant = self.constants.get(local.identifier_index as usize)?;
                let identifier =
                    if let Value::Concrete(ConcreteValue::String(identifier)) = constant {
                        identifier
                    } else {
                        return None;
                    };

                if identifier == identifier_text {
                    Some(index as u16)
                } else {
                    None
                }
            })
            .ok_or(CompileError::UndeclaredVariable {
                identifier: identifier_text.to_string(),
                position: self.current_position,
            })
    }

    fn declare_local(
        &mut self,
        identifier: &str,
        register_index: u16,
        r#type: Type,
        is_mutable: bool,
        scope: Scope,
    ) -> (u16, u16) {
        info!("Declaring local {identifier}");

        let identifier = Value::Concrete(ConcreteValue::string(identifier));
        let identifier_index = self.push_or_get_constant(identifier);
        let local_index = self.locals.len() as u16;

        self.locals.push((
            Local::new(identifier_index, register_index, is_mutable, scope),
            r#type,
        ));

        (local_index, identifier_index)
    }

    fn get_identifier(&self, local_index: u16) -> Option<String> {
        self.locals
            .get(local_index as usize)
            .and_then(|(local, _)| {
                self.constants
                    .get(local.identifier_index as usize)
                    .map(|value| value.to_string())
            })
    }

    fn push_or_get_constant(&mut self, value: Value) -> u16 {
        if let Some(index) = self
            .constants
            .iter()
            .position(|constant| constant == &value)
        {
            index as u16
        } else {
            let index = self.constants.len() as u16;

            self.constants.push(value);

            index
        }
    }

    fn allow(&mut self, allowed: Token) -> Result<bool, CompileError> {
        if self.current_token == allowed {
            self.advance()?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), CompileError> {
        if self.current_token == expected {
            self.advance()
        } else {
            Err(CompileError::ExpectedToken {
                expected: expected.kind(),
                found: self.current_token.to_owned(),
                position: self.current_position,
            })
        }
    }

    fn get_last_operation(&self) -> Option<Operation> {
        self.instructions
            .last()
            .map(|(instruction, _, _)| instruction.operation())
    }

    fn get_last_operations<const COUNT: usize>(&self) -> Option<[Operation; COUNT]> {
        let mut n_operations = [Operation::RETURN; COUNT];

        for (nth, operation) in n_operations.iter_mut().rev().zip(
            self.instructions
                .iter()
                .rev()
                .map(|(instruction, _, _)| instruction.operation()),
        ) {
            *nth = operation;
        }

        Some(n_operations)
    }

    fn get_last_instruction_type(&self) -> Type {
        self.instructions
            .last()
            .map(|(_, r#type, _)| r#type.clone())
            .unwrap_or(Type::None)
    }

    fn get_register_type(&self, register_index: u16) -> Result<Type, CompileError> {
        if let Some((_, r#type)) = self
            .locals
            .iter()
            .find(|(local, _)| local.register_index == register_index)
        {
            return Ok(r#type.clone());
        }

        for (instruction, r#type, _) in &self.instructions {
            if !instruction.yields_value() {
                continue;
            }

            let operation = instruction.operation();

            if let Operation::LOAD_LIST = operation {
                let LoadList { start_register, .. } = LoadList::from(*instruction);
                let item_type = self.get_register_type(start_register)?;

                return Ok(Type::List(Box::new(item_type)));
            }

            if let Operation::LOAD_SELF = operation {
                return Ok(Type::SelfFunction);
            }

            if instruction.yields_value() {
                return Ok(r#type.clone());
            }
        }

        Err(CompileError::CannotResolveRegisterType {
            register_index: register_index as usize,
            position: self.current_position,
        })
    }

    /// Updates [`Self::type`] with the given [Type] as `return_type`.
    ///
    /// If [`Self::type`] is already set, it will check if the given [Type] is compatible.
    fn update_return_type(&mut self, new_return_type: Type) -> Result<(), CompileError> {
        if self.r#type.return_type != Type::None {
            self.r#type
                .return_type
                .check(&new_return_type)
                .map_err(|conflict| CompileError::ReturnTypeConflict {
                    conflict,
                    position: self.previous_position,
                })?;
        }

        self.r#type.return_type = new_return_type;

        Ok(())
    }

    fn emit_instruction(&mut self, instruction: Instruction, r#type: Type, position: Span) {
        debug!(
            "Emitting {} at {}",
            instruction.operation().to_string(),
            position.to_string()
        );

        if instruction.yields_value() {
            let destination = instruction.a_field() as usize;

            self.stack_size = (destination + 1).max(self.stack_size);
        }

        self.instructions.push((instruction, r#type, position));
    }

    fn emit_constant(
        &mut self,
        constant: ConcreteValue,
        position: Span,
    ) -> Result<(), CompileError> {
        let r#type = constant.r#type();
        let constant_index = self.push_or_get_constant(Value::Concrete(constant));
        let destination = self.next_register();
        let load_constant = Instruction::load_constant(destination, constant_index, false);

        self.emit_instruction(load_constant, r#type, position);

        Ok(())
    }

    fn parse_boolean(&mut self) -> Result<(), CompileError> {
        let position = self.current_position;

        if let Token::Boolean(text) = self.current_token {
            self.advance()?;

            let boolean = text.parse::<bool>().unwrap();
            let destination = self.next_register();
            let load_boolean = Instruction::load_boolean(destination, boolean, false);

            self.emit_instruction(load_boolean, Type::Boolean, position);

            Ok(())
        } else {
            Err(CompileError::ExpectedToken {
                expected: TokenKind::Boolean,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_byte(&mut self) -> Result<(), CompileError> {
        let position = self.current_position;

        if let Token::Byte(text) = self.current_token {
            self.advance()?;

            let byte = u8::from_str_radix(&text[2..], 16)
                .map_err(|error| CompileError::ParseIntError { error, position })?;
            let value = ConcreteValue::Byte(byte);

            self.emit_constant(value, position)?;

            Ok(())
        } else {
            Err(CompileError::ExpectedToken {
                expected: TokenKind::Byte,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_character(&mut self) -> Result<(), CompileError> {
        let position = self.current_position;

        if let Token::Character(character) = self.current_token {
            self.advance()?;

            let value = ConcreteValue::Character(character);

            self.emit_constant(value, position)?;

            Ok(())
        } else {
            Err(CompileError::ExpectedToken {
                expected: TokenKind::Character,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_float(&mut self) -> Result<(), CompileError> {
        let position = self.current_position;

        if let Token::Float(text) = self.current_token {
            self.advance()?;

            let float = text
                .parse::<f64>()
                .map_err(|error| CompileError::ParseFloatError {
                    error,
                    position: self.previous_position,
                })?;
            let value = ConcreteValue::Float(float);

            self.emit_constant(value, position)?;

            Ok(())
        } else {
            Err(CompileError::ExpectedToken {
                expected: TokenKind::Float,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_integer(&mut self) -> Result<(), CompileError> {
        let position = self.current_position;

        if let Token::Integer(text) = self.current_token {
            self.advance()?;

            let mut integer_value = 0_i64;

            for digit in text.chars() {
                let digit = if let Some(digit) = digit.to_digit(10) {
                    digit as i64
                } else {
                    continue;
                };

                integer_value = integer_value * 10 + digit;
            }

            let value = ConcreteValue::Integer(integer_value);

            self.emit_constant(value, position)?;

            Ok(())
        } else {
            Err(CompileError::ExpectedToken {
                expected: TokenKind::Integer,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_string(&mut self) -> Result<(), CompileError> {
        let position = self.current_position;

        if let Token::String(text) = self.current_token {
            self.advance()?;

            let value = ConcreteValue::string(text);

            self.emit_constant(value, position)?;

            Ok(())
        } else {
            Err(CompileError::ExpectedToken {
                expected: TokenKind::String,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_grouped(&mut self) -> Result<(), CompileError> {
        self.allow(Token::LeftParenthesis)?;
        self.parse_expression()?;
        self.expect(Token::RightParenthesis)?;

        Ok(())
    }

    fn parse_unary(&mut self) -> Result<(), CompileError> {
        let operator = self.current_token;
        let operator_position = self.current_position;

        self.advance()?;
        self.parse_expression()?;

        let (previous_instruction, previous_type, previous_position) =
            self.instructions.pop().unwrap();
        let (argument, push_back) = self.handle_binary_argument(&previous_instruction)?;

        if push_back {
            self.instructions.push((
                previous_instruction,
                previous_type.clone(),
                previous_position,
            ))
        }

        let destination = self.next_register();
        let type_code = match previous_type {
            Type::Boolean => TypeCode::BOOLEAN,
            Type::Byte => TypeCode::BYTE,
            Type::Character => TypeCode::CHARACTER,
            Type::Float => TypeCode::FLOAT,
            Type::Integer => TypeCode::INTEGER,
            Type::String => TypeCode::STRING,
            _ => match operator {
                Token::Minus => {
                    return Err(CompileError::CannotNegateType {
                        argument_type: previous_type,
                        position: previous_position,
                    });
                }
                Token::Bang => {
                    return Err(CompileError::CannotNotType {
                        argument_type: previous_type,
                        position: previous_position,
                    });
                }
                _ => {
                    return Err(CompileError::ExpectedTokenMultiple {
                        expected: &[TokenKind::Bang, TokenKind::Minus],
                        found: operator.to_owned(),
                        position: operator_position,
                    });
                }
            },
        };
        let instruction = match operator {
            Token::Bang => Instruction::not(destination, argument),
            Token::Minus => Instruction::negate(destination, argument, type_code),
            _ => unreachable!(),
        };

        self.emit_instruction(instruction, previous_type, operator_position);

        Ok(())
    }

    fn handle_binary_argument(
        &mut self,
        instruction: &Instruction,
    ) -> Result<(Operand, bool), CompileError> {
        let (argument, push_back) = match instruction.operation() {
            Operation::LOAD_CONSTANT => (Operand::Constant(instruction.b_field()), false),
            Operation::POINT => (instruction.b_as_operand(), false),
            Operation::LOAD_BOOLEAN
            | Operation::LOAD_LIST
            | Operation::LOAD_SELF
            | Operation::ADD
            | Operation::SUBTRACT
            | Operation::MULTIPLY
            | Operation::DIVIDE
            | Operation::MODULO
            | Operation::EQUAL
            | Operation::LESS
            | Operation::LESS_EQUAL
            | Operation::NEGATE
            | Operation::NOT
            | Operation::CALL => (Operand::Register(instruction.a_field()), true),
            Operation::CALL_NATIVE => {
                let function = NativeFunction::from(instruction.b_field());

                if function.returns_value() {
                    (Operand::Register(instruction.a_field()), true)
                } else {
                    return Err(CompileError::ExpectedExpression {
                        found: self.previous_token.to_owned(),
                        position: self.previous_position,
                    });
                }
            }
            _ => {
                return Err(CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                });
            }
        };

        Ok((argument, push_back))
    }

    fn parse_math_binary(&mut self) -> Result<(), CompileError> {
        let (left_instruction, left_type, left_position) =
            self.instructions
                .pop()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;
        let (left, push_back_left) = self.handle_binary_argument(&left_instruction)?;
        let left_is_mutable_local = if left_instruction.operation() == Operation::POINT {
            let Point { to, .. } = Point::from(left_instruction);

            self.locals
                .get(to.index() as usize)
                .map(|(local, _)| local.is_mutable)
                .unwrap_or(false)
        } else {
            false
        };

        if push_back_left {
            self.instructions
                .push((left_instruction, left_type.clone(), left_position));
        }

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(&operator);
        let is_assignment = matches!(
            operator,
            Token::PlusEqual
                | Token::MinusEqual
                | Token::StarEqual
                | Token::SlashEqual
                | Token::PercentEqual
        );

        check_math_type(&left_type, operator, &left_position)?;

        let left_type_code = match left_type {
            Type::Boolean => TypeCode::BOOLEAN,
            Type::Byte => TypeCode::BYTE,
            Type::Character => TypeCode::CHARACTER,
            Type::Float => TypeCode::FLOAT,
            Type::Integer => TypeCode::INTEGER,
            Type::String => TypeCode::STRING,
            _ => unreachable!(),
        };

        if is_assignment && !left_is_mutable_local {
            return Err(CompileError::ExpectedMutableVariable {
                found: self.previous_token.to_owned(),
                position: left_position,
            });
        }

        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        let (right_instruction, right_type, right_position) = self.instructions.pop().unwrap();
        let (right, push_back_right) = self.handle_binary_argument(&right_instruction)?;

        check_math_type(&right_type, operator, &right_position)?;
        check_math_types(
            &left_type,
            &left_position,
            operator,
            &right_type,
            &right_position,
        )?;

        let right_type_code = match right_type {
            Type::Boolean => TypeCode::BOOLEAN,
            Type::Byte => TypeCode::BYTE,
            Type::Character => TypeCode::CHARACTER,
            Type::Float => TypeCode::FLOAT,
            Type::Integer => TypeCode::INTEGER,
            Type::String => TypeCode::STRING,
            _ => unreachable!(),
        };

        if push_back_right {
            self.instructions
                .push((right_instruction, right_type, right_position));
        }

        let r#type = if is_assignment {
            Type::None
        } else if left_type == Type::Character {
            Type::String
        } else {
            left_type.clone()
        };
        let destination = if is_assignment {
            match left {
                Operand::Register(register) => register,
                Operand::Constant(_) => self.next_register(),
            }
        } else {
            self.next_register()
        };
        let instruction = match operator {
            Token::Plus | Token::PlusEqual => {
                Instruction::add(destination, left, left_type_code, right, right_type_code)
            }
            Token::Minus | Token::MinusEqual => {
                Instruction::subtract(destination, left, left_type_code, right, right_type_code)
            }
            Token::Star | Token::StarEqual => {
                Instruction::multiply(destination, left, left_type_code, right, right_type_code)
            }
            Token::Slash | Token::SlashEqual => {
                Instruction::divide(destination, left, left_type_code, right, right_type_code)
            }
            Token::Percent | Token::PercentEqual => {
                Instruction::modulo(destination, left, left_type_code, right, right_type_code)
            }
            _ => {
                return Err(CompileError::ExpectedTokenMultiple {
                    expected: &[
                        TokenKind::Plus,
                        TokenKind::PlusEqual,
                        TokenKind::Minus,
                        TokenKind::MinusEqual,
                        TokenKind::Star,
                        TokenKind::StarEqual,
                        TokenKind::Slash,
                        TokenKind::SlashEqual,
                        TokenKind::Percent,
                        TokenKind::PercentEqual,
                    ],
                    found: operator.to_owned(),
                    position: operator_position,
                });
            }
        };

        self.emit_instruction(instruction, r#type, operator_position);

        Ok(())
    }

    fn parse_comparison_binary(&mut self) -> Result<(), CompileError> {
        if let Some(
            [
                Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL,
                _,
                _,
            ],
        ) = self.get_last_operations()
        {
            return Err(CompileError::ComparisonChain {
                position: self.current_position,
            });
        }

        let (left_instruction, left_type, left_position) =
            self.instructions
                .pop()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;
        let (left, push_back_left) = self.handle_binary_argument(&left_instruction)?;
        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(&operator);

        // TODO: Check if the left type is a valid type for comparison

        let left_type_code = match left_type {
            Type::Boolean => TypeCode::BOOLEAN,
            Type::Byte => TypeCode::BYTE,
            Type::Character => TypeCode::CHARACTER,
            Type::Float => TypeCode::FLOAT,
            Type::Integer => TypeCode::INTEGER,
            Type::String => TypeCode::STRING,
            _ => unreachable!(),
        };

        if push_back_left {
            self.instructions
                .push((left_instruction, left_type, left_position));
        }

        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        let (right_instruction, right_type, right_position) =
            self.instructions
                .pop()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;
        let (right, push_back_right) = self.handle_binary_argument(&right_instruction)?;

        // TODO: Check if the right type is a valid type for comparison
        // TODO: Check if the left and right types are compatible

        let right_type_code = match right_type {
            Type::Boolean => TypeCode::BOOLEAN,
            Type::Byte => TypeCode::BYTE,
            Type::Character => TypeCode::CHARACTER,
            Type::Float => TypeCode::FLOAT,
            Type::Integer => TypeCode::INTEGER,
            Type::String => TypeCode::STRING,
            _ => unreachable!(),
        };

        if push_back_right {
            self.instructions
                .push((right_instruction, right_type, right_position));
        }

        let destination = self.next_register();
        let comparison = match operator {
            Token::DoubleEqual => {
                Instruction::equal(true, left, left_type_code, right, right_type_code)
            }
            Token::BangEqual => {
                Instruction::equal(false, left, left_type_code, right, right_type_code)
            }
            Token::Less => Instruction::less(true, left, left_type_code, right, right_type_code),
            Token::LessEqual => {
                Instruction::less_equal(true, left, left_type_code, right, right_type_code)
            }
            Token::Greater => {
                Instruction::less_equal(false, left, left_type_code, right, right_type_code)
            }
            Token::GreaterEqual => {
                Instruction::less(false, left, left_type_code, right, right_type_code)
            }
            _ => {
                return Err(CompileError::ExpectedTokenMultiple {
                    expected: &[
                        TokenKind::DoubleEqual,
                        TokenKind::BangEqual,
                        TokenKind::Less,
                        TokenKind::LessEqual,
                        TokenKind::Greater,
                        TokenKind::GreaterEqual,
                    ],
                    found: operator.to_owned(),
                    position: operator_position,
                });
            }
        };
        let jump = Instruction::jump(1, true);
        let load_true = Instruction::load_boolean(destination, true, true);
        let load_false = Instruction::load_boolean(destination, false, false);

        self.emit_instruction(comparison, Type::Boolean, operator_position);
        self.emit_instruction(jump, Type::None, operator_position);
        self.emit_instruction(load_true, Type::Boolean, operator_position);
        self.emit_instruction(load_false, Type::Boolean, operator_position);

        Ok(())
    }

    fn parse_logical_binary(&mut self) -> Result<(), CompileError> {
        let (last_instruction, last_type, last_position) =
            self.instructions
                .pop()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;
        let operand_register = if last_instruction.operation() == Operation::POINT {
            let Point { to, .. } = Point::from(last_instruction);
            let (local, _) = self.get_local(to.index())?;

            local.register_index
        } else if last_instruction.yields_value() {
            let register = last_instruction.a_field();

            self.instructions
                .push((last_instruction, last_type, last_position));

            register
        } else {
            self.instructions
                .push((last_instruction, last_type, last_position));

            self.next_register().saturating_sub(1)
        };
        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(&operator);
        let test_boolean = match operator {
            Token::DoubleAmpersand => true,
            Token::DoublePipe => false,
            _ => {
                return Err(CompileError::ExpectedTokenMultiple {
                    expected: &[TokenKind::DoubleAmpersand, TokenKind::DoublePipe],
                    found: operator.to_owned(),
                    position: operator_position,
                });
            }
        };
        let test = Instruction::test(operand_register, test_boolean);
        let jump = Instruction::jump(1, true);

        self.emit_instruction(test, Type::None, operator_position);
        self.emit_instruction(jump, Type::None, operator_position);
        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        let instructions_length = self.instructions.len();

        for (group_index, instructions) in self.instructions.rchunks_mut(3).enumerate().rev() {
            if instructions.len() < 3 {
                continue;
            }

            if !matches!(
                instructions[0].0.operation(),
                Operation::TEST | Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL
            ) || !matches!(instructions[1].0.operation(), Operation::JUMP)
            {
                continue;
            }

            let old_jump = &mut instructions[1].0;
            let jump_index = instructions_length - group_index * 3 - 1;
            let short_circuit_distance = (instructions_length - jump_index) as u16;

            *old_jump = Instruction::jump(short_circuit_distance, true);
        }

        Ok(())
    }

    fn parse_variable(&mut self) -> Result<(), CompileError> {
        let start_position = self.current_position;
        let identifier = if let Token::Identifier(text) = self.current_token {
            self.advance()?;

            text
        } else {
            return Err(CompileError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: self.current_token.to_owned(),
                position: start_position,
            });
        };
        let local_index = if let Ok(local_index) = self.get_local_index(identifier) {
            local_index
        } else if let Some(native_function) = NativeFunction::from_str(identifier) {
            return self.parse_call_native(native_function);
        } else if self.function_name.as_deref() == Some(identifier) && !self.is_main {
            let destination = self.next_register();
            let load_self = Instruction::load_self(destination, false);

            self.emit_instruction(load_self, Type::SelfFunction, start_position);

            return Ok(());
        } else {
            return Err(CompileError::UndeclaredVariable {
                identifier: identifier.to_string(),
                position: start_position,
            });
        };

        let (local, r#type) = self
            .get_local(local_index)
            .map(|(local, r#type)| (local, r#type.clone()))?;
        let is_mutable = local.is_mutable;
        let local_register_index = local.register_index;

        if !self.current_scope.contains(&local.scope) {
            return Err(CompileError::VariableOutOfScope {
                identifier: self.get_identifier(local_index).unwrap(),
                position: start_position,
                variable_scope: local.scope,
                access_scope: self.current_scope,
            });
        }

        if self.allow(Token::Equal)? {
            if !is_mutable {
                return Err(CompileError::CannotMutateImmutableVariable {
                    identifier: self.get_identifier(local_index).unwrap(),
                    position: start_position,
                });
            }

            self.parse_expression()?;

            if self
                .instructions
                .last()
                .is_some_and(|(instruction, _, _)| instruction.is_math())
            {
                let (math_instruction, _, _) = self.instructions.last_mut().unwrap();

                math_instruction.set_a_field(local_register_index);
            } else {
                let register = self.next_register() - 1;
                let point = Instruction::point(local_register_index, Operand::Register(register));

                self.emit_instruction(point, r#type, start_position);
            }

            return Ok(());
        }

        let destination = self.next_register();
        let point = Instruction::point(destination, Operand::Register(local_register_index));

        self.emit_instruction(point, r#type, self.previous_position);

        Ok(())
    }

    fn parse_type_from(&mut self, token: Token, position: Span) -> Result<Type, CompileError> {
        match token {
            Token::Bool => Ok(Type::Boolean),
            Token::FloatKeyword => Ok(Type::Float),
            Token::Int => Ok(Type::Integer),
            Token::Str => Ok(Type::String),
            _ => Err(CompileError::ExpectedTokenMultiple {
                expected: &[
                    TokenKind::Bool,
                    TokenKind::FloatKeyword,
                    TokenKind::Int,
                    TokenKind::Str,
                ],
                found: self.current_token.to_owned(),
                position,
            }),
        }
    }

    fn parse_block(&mut self) -> Result<(), CompileError> {
        self.advance()?;

        let starting_block = self.current_scope.block_index;

        self.block_index += 1;
        self.current_scope.begin(self.block_index);

        while !self.allow(Token::RightBrace)? && !self.is_eof() {
            self.parse(Precedence::None)?;
        }

        self.current_scope.end(starting_block);

        Ok(())
    }

    fn parse_list(&mut self) -> Result<(), CompileError> {
        let start = self.current_position.0;

        self.advance()?;

        let start_register = self.next_register();
        let mut item_type = Type::Any;

        while !self.allow(Token::RightBracket)? && !self.is_eof() {
            let expected_register = self.next_register();

            self.parse_expression()?;

            let actual_register = self.next_register() - 1;

            if item_type == Type::Any {
                item_type = self.get_last_instruction_type();
            }

            if expected_register < actual_register {
                let close = Instruction::from(Close {
                    from: expected_register,
                    to: actual_register,
                });

                self.emit_instruction(close, Type::None, self.current_position);
            }

            self.allow(Token::Comma)?;
        }

        let destination = self.next_register();
        let end = self.previous_position.1;
        let load_list = Instruction::load_list(destination, start_register, false);

        self.emit_instruction(load_list, Type::List(Box::new(item_type)), Span(start, end));

        Ok(())
    }

    fn parse_if(&mut self) -> Result<(), CompileError> {
        self.advance()?;
        self.parse_expression()?;

        if matches!(
            self.get_last_operations(),
            Some([
                Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL,
                Operation::JUMP,
                Operation::LOAD_BOOLEAN,
                Operation::LOAD_BOOLEAN
            ]),
        ) {
            self.instructions.pop();
            self.instructions.pop();
            self.instructions.pop();
        } else {
            let operand_register = self.next_register() - 1;
            let test = Instruction::test(operand_register, true);

            self.emit_instruction(test, Type::None, self.current_position);
        }

        let if_block_start = self.instructions.len();
        let if_block_start_position = self.current_position;

        if let Token::LeftBrace = self.current_token {
            self.parse_block()?;
        } else {
            return Err(CompileError::ExpectedTokenMultiple {
                expected: &[TokenKind::If, TokenKind::LeftBrace],
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        }

        let if_block_end = self.instructions.len();
        let mut if_block_distance = (if_block_end - if_block_start) as u16;
        let if_block_type = self.get_last_instruction_type();

        if let Token::Else = self.current_token {
            self.advance()?;

            if let Token::LeftBrace = self.current_token {
                self.parse_block()?;
            } else {
                return Err(CompileError::ExpectedTokenMultiple {
                    expected: &[TokenKind::If, TokenKind::LeftBrace],
                    found: self.current_token.to_owned(),
                    position: self.current_position,
                });
            }
        } else if if_block_type != Type::None {
            return Err(CompileError::IfMissingElse {
                position: Span(if_block_start_position.0, self.current_position.1),
            });
        }

        let else_block_end = self.instructions.len();
        let else_block_distance = (else_block_end - if_block_end) as u16;
        let else_block_type = self.get_last_instruction_type();

        if let Err(conflict) = if_block_type.check(&else_block_type) {
            return Err(CompileError::IfElseBranchMismatch {
                conflict,
                position: Span(if_block_start_position.0, self.current_position.1),
            });
        }

        match else_block_distance {
            0 => {}
            1 => {
                if let Some(Operation::LOAD_BOOLEAN | Operation::LOAD_CONSTANT) =
                    self.get_last_operation()
                {
                    let (loader, _, _) = self.instructions.last_mut().unwrap();

                    loader.set_c_field(true as u16);
                } else {
                    if_block_distance += 1;
                    let jump = Instruction::from(Jump {
                        offset: else_block_distance,
                        is_positive: true,
                    });

                    self.instructions
                        .insert(if_block_end, (jump, Type::None, self.current_position));
                }
            }
            2.. => {
                if_block_distance += 1;
                let jump = Instruction::from(Jump {
                    offset: else_block_distance,
                    is_positive: true,
                });

                self.instructions
                    .insert(if_block_end, (jump, Type::None, self.current_position));
            }
        }

        let jump = Instruction::jump(if_block_distance, true);

        self.instructions
            .insert(if_block_start, (jump, Type::None, if_block_start_position));
        control_flow_register_consolidation(self);

        Ok(())
    }

    fn parse_while(&mut self) -> Result<(), CompileError> {
        self.advance()?;

        let expression_start = self.instructions.len();

        self.parse_expression()?;

        if matches!(
            self.get_last_operations(),
            Some([
                Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL,
                Operation::JUMP,
                Operation::LOAD_BOOLEAN,
                Operation::LOAD_BOOLEAN
            ]),
        ) {
            self.instructions.pop();
            self.instructions.pop();
            self.instructions.pop();
        } else {
            let operand_register = self.next_register() - 1;
            let test = Instruction::test(operand_register, true);

            self.emit_instruction(test, Type::None, self.current_position);
        }

        let block_start = self.instructions.len();

        self.parse_block()?;

        let block_end = self.instructions.len();
        let jump_distance = (block_end - block_start + 1) as u16;
        let jump = Instruction::from(Jump {
            offset: jump_distance,
            is_positive: true,
        });

        self.instructions
            .insert(block_start, (jump, Type::None, self.current_position));

        let jump_back_distance = (block_end - expression_start + 1) as u16;
        let jump_back = Instruction::from(Jump {
            offset: jump_back_distance,
            is_positive: false,
        });

        self.emit_instruction(jump_back, Type::None, self.current_position);

        Ok(())
    }

    fn parse_call_native(&mut self, function: NativeFunction) -> Result<(), CompileError> {
        let start = self.previous_position.0;
        let start_register = self.next_register();

        self.expect(Token::LeftParenthesis)?;

        while !self.allow(Token::RightParenthesis)? {
            let expected_register = self.next_register();

            self.parse_expression()?;

            let actual_register = self.next_register() - 1;
            let registers_to_close = actual_register - expected_register;

            if registers_to_close > 0 {
                let close = Instruction::from(Close {
                    from: expected_register,
                    to: actual_register,
                });

                self.emit_instruction(close, Type::None, self.current_position);
            }

            self.allow(Token::Comma)?;
        }

        let end = self.previous_position.1;
        let destination = self.next_register();
        let argument_count = destination - start_register;
        let return_type = function.r#type().return_type;
        let call_native = Instruction::from(CallNative {
            destination,
            function,
            argument_count,
        });

        self.emit_instruction(call_native, return_type, Span(start, end));

        Ok(())
    }

    fn parse_semicolon(&mut self) -> Result<(), CompileError> {
        self.advance()?;
        self.parse(Precedence::None)
    }

    fn parse_expression(&mut self) -> Result<(), CompileError> {
        self.parse(Precedence::None)?;

        let expression_type = self.get_last_instruction_type();

        if expression_type == Type::None || self.instructions.is_empty() {
            return Err(CompileError::ExpectedExpression {
                found: self.previous_token.to_owned(),
                position: self.current_position,
            });
        }

        Ok(())
    }

    fn parse_sub_expression(&mut self, precedence: &Precedence) -> Result<(), CompileError> {
        self.parse(precedence.increment())?;

        let expression_type = self.get_last_instruction_type();

        if expression_type == Type::None || self.instructions.is_empty() {
            return Err(CompileError::ExpectedExpression {
                found: self.previous_token.to_owned(),
                position: self.current_position,
            });
        }

        Ok(())
    }

    fn parse_return_statement(&mut self) -> Result<(), CompileError> {
        let start = self.current_position.0;

        self.advance()?;

        let should_return_value =
            if matches!(self.current_token, Token::Semicolon | Token::RightBrace) {
                self.update_return_type(Type::None)?;

                false
            } else {
                self.parse_expression()?;

                let expression_type = self.get_last_instruction_type();

                self.update_return_type(expression_type)?;

                true
            };
        let end = self.current_position.1;
        let return_register = self.next_register() - 1;
        let r#return = Instruction::from(Return {
            should_return_value,
            return_register,
        });

        self.emit_instruction(r#return, Type::None, Span(start, end));

        let instruction_length = self.instructions.len();

        for (index, (instruction, _, _)) in self.instructions.iter_mut().enumerate() {
            if instruction.operation() == Operation::JUMP {
                let Jump {
                    offset,
                    is_positive,
                } = Jump::from(*instruction);
                let offset = offset as usize;

                if is_positive && offset + index == instruction_length - 1 {
                    *instruction = Instruction::jump((offset + 1) as u16, true);
                }
            }
        }

        Ok(())
    }

    fn parse_implicit_return(&mut self) -> Result<(), CompileError> {
        if matches!(self.get_last_operation(), Some(Operation::RETURN))
            || matches!(
                self.get_last_operations(),
                Some([Operation::RETURN, Operation::JUMP])
            )
        {
            // Do nothing if the last instruction is a return or a return followed by a jump
        } else if self.allow(Token::Semicolon)? {
            let r#return = Instruction::r#return(false, 0);

            self.emit_instruction(r#return, Type::None, self.current_position);
        } else {
            let (previous_expression_type, previous_register) = self
                .instructions
                .last()
                .map(|(instruction, r#type, _)| {
                    if instruction.yields_value() {
                        (r#type.clone(), instruction.a_field())
                    } else {
                        (Type::None, 0)
                    }
                })
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;

            let should_return_value = previous_expression_type != Type::None;
            let r#return = Instruction::r#return(should_return_value, previous_register);

            self.update_return_type(previous_expression_type.clone())?;
            self.emit_instruction(r#return, Type::None, self.current_position);
        }

        let instruction_length = self.instructions.len();

        for (index, (instruction, _, _)) in self.instructions.iter_mut().enumerate() {
            if instruction.operation() == Operation::JUMP {
                let Jump {
                    offset,
                    is_positive,
                } = Jump::from(*instruction);
                let offset = offset as usize;

                if is_positive && offset + index == instruction_length - 1 {
                    *instruction = Instruction::jump((offset + 1) as u16, true);
                }
            }
        }

        Ok(())
    }

    fn parse_let_statement(&mut self) -> Result<(), CompileError> {
        self.advance()?;

        let is_mutable = self.allow(Token::Mut)?;
        let position = self.current_position;
        let identifier = if let Token::Identifier(text) = self.current_token {
            self.advance()?;

            text
        } else {
            return Err(CompileError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: self.current_token.to_owned(),
                position,
            });
        };
        let explicit_type = if self.allow(Token::Colon)? {
            self.advance()?;

            let r#type = self.parse_type_from(self.current_token, self.current_position)?;

            Some(r#type)
        } else {
            None
        };

        self.expect(Token::Equal)?;
        self.parse_expression()?;

        let register_index = self.next_register() - 1;
        let r#type = if let Some(r#type) = explicit_type {
            r#type
        } else {
            self.get_register_type(register_index)?
        };

        self.declare_local(
            identifier,
            register_index,
            r#type,
            is_mutable,
            self.current_scope,
        );

        // The last instruction is now an assignment, so it should not yield a value
        self.instructions.last_mut().unwrap().1 = Type::None;

        Ok(())
    }

    fn parse_function(&mut self) -> Result<(), CompileError> {
        let function_start = self.current_position.0;

        self.advance()?;

        let identifier = if let Token::Identifier(text) = self.current_token {
            self.advance()?;

            Some(text)
        } else {
            None
        };

        let mut function_compiler = if self.current_token == Token::LeftParenthesis {
            let function_name = identifier.map(DustString::from);

            Compiler::new(self.lexer, function_name, false)? // This will consume the parenthesis
        } else {
            return Err(CompileError::ExpectedToken {
                expected: TokenKind::LeftParenthesis,
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        };

        function_compiler.prototype_index = self.prototypes.len() as u16;

        let mut value_parameters: Vec<(u16, Type)> = Vec::with_capacity(3);

        while !function_compiler.allow(Token::RightParenthesis)? {
            let is_mutable = function_compiler.allow(Token::Mut)?;
            let parameter = if let Token::Identifier(text) = function_compiler.current_token {
                function_compiler.advance()?;

                text
            } else {
                return Err(CompileError::ExpectedToken {
                    expected: TokenKind::Identifier,
                    found: function_compiler.current_token.to_owned(),
                    position: function_compiler.current_position,
                });
            };

            function_compiler.expect(Token::Colon)?;

            let r#type = function_compiler.parse_type_from(
                function_compiler.current_token,
                function_compiler.current_position,
            )?;

            function_compiler.advance()?;

            let local_register_index = function_compiler.next_register();
            let (_, identifier_index) = function_compiler.declare_local(
                parameter,
                local_register_index,
                r#type.clone(),
                is_mutable,
                function_compiler.current_scope,
            );

            value_parameters.push((identifier_index, r#type));
            function_compiler.allow(Token::Comma)?;

            function_compiler.minimum_register += 1;
        }

        let return_type = if function_compiler.allow(Token::ArrowThin)? {
            let r#type = function_compiler.parse_type_from(
                function_compiler.current_token,
                function_compiler.current_position,
            )?;

            function_compiler.advance()?;

            r#type
        } else {
            Type::None
        };
        let function_type = FunctionType {
            type_parameters: Vec::with_capacity(0),
            value_parameters,
            return_type,
        };

        function_compiler.r#type = function_type.clone();

        function_compiler.expect(Token::LeftBrace)?;
        function_compiler.compile()?;
        function_compiler.expect(Token::RightBrace)?;

        self.previous_token = function_compiler.previous_token;
        self.previous_position = function_compiler.previous_position;
        self.current_token = function_compiler.current_token;
        self.current_position = function_compiler.current_position;

        self.lexer.skip_to(self.current_position.1);

        let function_end = function_compiler.previous_position.1;
        let prototype_index = function_compiler.prototype_index;
        let chunk = function_compiler.finish();
        let destination = self.next_register();

        self.prototypes.push(Arc::new(chunk));

        if let Some(identifier) = identifier {
            self.declare_local(
                identifier,
                destination,
                Type::function(function_type.clone()),
                false,
                self.current_scope,
            );
        }

        let load_function = Instruction::load_function(destination, prototype_index, false);

        self.emit_instruction(
            load_function,
            Type::function(function_type),
            Span(function_start, function_end),
        );

        Ok(())
    }

    fn parse_call(&mut self) -> Result<(), CompileError> {
        let start = self.current_position.0;

        self.advance()?;

        let (last_instruction, last_instruction_type, _) =
            self.instructions
                .last()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;

        if !matches!(
            last_instruction_type,
            Type::Function(_) | Type::SelfFunction
        ) {
            return Err(CompileError::ExpectedFunction {
                found: self.previous_token.to_owned(),
                actual_type: last_instruction_type.clone(),
                position: self.previous_position,
            });
        }

        let function_register = last_instruction.a_field();
        let function_return_type = match last_instruction_type {
            Type::Function(function_type) => function_type.return_type.clone(),
            Type::SelfFunction => self.r#type.return_type.clone(),
            _ => {
                return Err(CompileError::ExpectedFunction {
                    found: self.previous_token.to_owned(),
                    actual_type: last_instruction_type.clone(),
                    position: self.previous_position,
                });
            }
        };
        let is_recursive = last_instruction_type == &Type::SelfFunction;

        let mut argument_count = 0;

        while !self.allow(Token::RightParenthesis)? {
            let expected_register = self.next_register();

            self.parse_expression()?;

            let actual_register = self.next_register() - 1;
            let registers_to_close = (actual_register - expected_register).saturating_sub(1);

            if registers_to_close > 0 {
                let close = Instruction::from(Close {
                    from: expected_register,
                    to: actual_register,
                });

                self.emit_instruction(close, Type::None, self.current_position);
            }

            argument_count += registers_to_close + 1;

            self.allow(Token::Comma)?;
        }

        let end = self.current_position.1;
        let destination = self.next_register();
        let call = Instruction::call(destination, function_register, argument_count, is_recursive);

        self.emit_instruction(call, function_return_type, Span(start, end));

        Ok(())
    }

    fn expect_expression(&mut self) -> Result<(), CompileError> {
        Err(CompileError::ExpectedExpression {
            found: self.current_token.to_owned(),
            position: self.current_position,
        })
    }

    fn parse(&mut self, precedence: Precedence) -> Result<(), CompileError> {
        if let Some(prefix_parser) = ParseRule::from(&self.current_token).prefix {
            debug!(
                "{} is prefix with precedence {precedence}",
                self.current_token.to_string(),
            );

            prefix_parser(self)?;
        }

        let mut infix_rule = ParseRule::from(&self.current_token);

        while precedence <= infix_rule.precedence {
            if let Some(infix_parser) = infix_rule.infix {
                debug!(
                    "{} is infix with precedence {precedence}",
                    self.current_token.to_string(),
                );

                if self.current_token == Token::Equal {
                    return Err(CompileError::InvalidAssignmentTarget {
                        found: self.current_token.to_owned(),
                        position: self.current_position,
                    });
                }

                infix_parser(self)?;
            } else {
                break;
            }

            infix_rule = ParseRule::from(&self.current_token);
        }

        Ok(())
    }
}
