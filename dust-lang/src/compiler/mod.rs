//! The Dust compiler and its accessories.
//!
//! This module provides two compilation options:
//! - [`compile`] is a simple function that borrows a string and returns a chunk, handling
//!   compilation and turning any resulting error into a [`DustError`], which can easily display a
//!   detailed report. The main chunk will be named "main".
//! - [`Compiler`] is created with a [`Lexer`] and potentially emits a [`CompileError`] or
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
mod parse_rule;
mod type_checks;

pub use error::CompileError;
use parse_rule::{ParseRule, Precedence};
use tracing::{Level, debug, info, span};
use type_checks::{check_math_type, check_math_types};

use std::{mem::replace, sync::Arc};

use crate::{
    Chunk, DustError, DustString, FunctionType, Instruction, Lexer, Local, NativeFunction, Operand,
    Operation, Scope, Span, Token, TokenKind, Type,
    instruction::{Jump, LoadFunction, Move, Return, TypeCode},
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
/// assert_eq!(chunk.instructions.len(), 6);
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

    /// Character constants that have been compiled. These are assigned to the chunk when
    /// [`Compiler::finish`] is called.
    character_constants: Vec<char>,

    /// Float constants that have been compiled. These are assigned to the chunk when
    /// [`Compiler::finish`] is called.
    float_constants: Vec<f64>,

    /// Integer constants that have been compiled. These are assigned to the chunk when
    /// [`Compiler::finish`] is called.
    integer_constants: Vec<i64>,

    /// String constants that have been compiled. These are assigned to the chunk when
    /// [`Compiler::finish`] is called.
    string_constants: Vec<DustString>,

    /// Block-local variables and their types. The locals are assigned to the chunk when
    /// [`Compiler::finish`] is called. The types are discarded after compilation.
    locals: Vec<Local>,

    /// Prototypes that have been compiled. These are assigned to the chunk when
    /// [`Compiler::finish`] is called.
    prototypes: Vec<Arc<Chunk>>,

    /// Lists of arguments for each function call. The integers represent the register of each
    /// argument. Note that the type of each argument is not stored, so the caller must check the
    /// function's type to determine the type of each argument.
    argument_lists: Vec<(Vec<u16>, Vec<Type>)>,

    /// The first boolean register index that the compiler should use. This is used to avoid reusing
    /// the registers that are used for the function's arguments.
    minimum_boolean_register: u16,

    /// The first byte register index that the compiler should use. This is used to avoid reusing
    /// the registers that are used for the function's arguments.
    minimum_byte_register: u16,

    /// The first character register index that the compiler should use. This is used to avoid
    /// reusing the registers that are used for the function's arguments.
    minimum_character_register: u16,

    /// The first float register index that the compiler should use. This is used to avoid reusing
    /// the registers that are used for the function's arguments.
    minimum_float_register: u16,

    /// The first integer register index that the compiler should use. This is used to avoid reusing
    /// the registers that are used for the function's arguments.
    minimum_integer_register: u16,

    /// The first string register index that the compiler should use. This is used to avoid reusing
    /// the registers that are used for the function's arguments.
    minimum_string_register: u16,

    /// The first list register index that the compiler should use. This is used to avoid reusing
    /// the registers that are used for the function's arguments.
    minimum_list_register: u16,

    /// The first function register index that the compiler should use. This is used to avoid
    /// reusing the registers that are used for the function's arguments.
    minimum_function_register: u16,

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
            r#type: FunctionType::default(),
            instructions: Vec::new(),
            character_constants: Vec::new(),
            float_constants: Vec::new(),
            integer_constants: Vec::new(),
            string_constants: Vec::new(),
            locals: Vec::new(),
            prototypes: Vec::new(),
            argument_lists: Vec::new(),
            lexer,
            minimum_byte_register: 0,
            minimum_boolean_register: 0,
            minimum_character_register: 0,
            minimum_float_register: 0,
            minimum_integer_register: 0,
            minimum_string_register: 0,
            minimum_list_register: 0,
            minimum_function_register: 0,
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

        while !matches!(self.current_token, Token::Eof | Token::RightBrace) {
            self.parse(Precedence::None)?;
        }

        self.parse_implicit_return()?;

        info!("End chunk");

        Ok(())
    }

    /// Creates a new chunk with the compiled data, optionally assigning a name to the chunk.
    ///
    /// Note for maintainers: Do not give a name when compiling functions, only the main chunk. This
    /// will allow [`Compiler::function_name`] to be both the name used for recursive calls and the
    /// name of the function when it is compiled. The name can later be seen in the VM's call stack.
    pub fn finish(mut self) -> Chunk {
        if self.instructions.is_empty() {
            let r#return = Instruction::r#return(false, 0, TypeCode::NONE);

            self.emit_instruction(r#return, Type::None, self.current_position);
        }

        let boolean_register_count = self.next_boolean_register();
        let byte_register_count = self.next_byte_register();
        let character_register_count = self.next_character_register();
        let float_register_count = self.next_float_register();
        let integer_register_count = self.next_integer_register();
        let string_register_count = self.next_string_register();
        let list_register_count = self.next_list_register();
        let function_register_count = self.next_function_register();
        let (instructions, positions): (Vec<Instruction>, Vec<Span>) = self
            .instructions
            .into_iter()
            .map(|(instruction, _, position)| (instruction, position))
            .unzip();

        Chunk {
            name: self.function_name,
            r#type: self.r#type,
            instructions,
            positions,
            character_constants: self.character_constants,
            float_constants: self.float_constants,
            integer_constants: self.integer_constants,
            string_constants: self.string_constants,
            locals: self.locals,
            prototypes: self.prototypes,
            argument_lists: self.argument_lists,
            boolean_register_count,
            byte_register_count,
            character_register_count,
            float_register_count,
            integer_register_count,
            string_register_count,
            list_register_count,
            function_register_count,
            prototype_index: self.prototype_index,
        }
    }

    fn is_eof(&self) -> bool {
        matches!(self.current_token, Token::Eof)
    }

    fn next_boolean_register(&self) -> u16 {
        self.instructions
            .iter()
            .rev()
            .find_map(|(instruction, _, _)| {
                if instruction.operation() == Operation::LOAD_ENCODED
                    && instruction.b_type() == TypeCode::BOOLEAN
                {
                    return Some(instruction.a_field() + 1);
                }

                None
            })
            .unwrap_or(self.minimum_boolean_register)
    }

    fn next_byte_register(&self) -> u16 {
        self.instructions
            .iter()
            .rev()
            .find_map(|(instruction, r#type, _)| {
                if *r#type == Type::Byte
                    || (instruction.operation() == Operation::LOAD_ENCODED
                        && instruction.b_type() == TypeCode::BYTE)
                {
                    return Some(instruction.a_field() + 1);
                }

                None
            })
            .unwrap_or(self.minimum_byte_register)
    }

    fn next_character_register(&self) -> u16 {
        self.instructions
            .iter()
            .rev()
            .find_map(|(instruction, r#type, _)| {
                if instruction.b_type() == TypeCode::CHARACTER
                    && r#type == &Type::Character
                    && instruction.yields_value()
                {
                    Some(instruction.a_field() + 1)
                } else {
                    None
                }
            })
            .unwrap_or(self.minimum_character_register)
    }

    fn next_float_register(&self) -> u16 {
        self.instructions
            .iter()
            .rev()
            .find_map(|(instruction, r#type, _)| {
                if *r#type == Type::Float
                    || (instruction.operation() == Operation::LOAD_CONSTANT
                        && instruction.b_type() == TypeCode::FLOAT)
                {
                    Some(instruction.a_field() + 1)
                } else {
                    None
                }
            })
            .unwrap_or(self.minimum_float_register)
    }

    fn next_integer_register(&self) -> u16 {
        self.instructions
            .iter()
            .rev()
            .find_map(|(instruction, r#type, _)| {
                if (instruction.operation() == Operation::LOAD_CONSTANT
                    && instruction.b_type() == TypeCode::INTEGER)
                    || instruction.yields_value() && r#type == &Type::Integer
                {
                    Some(instruction.a_field() + 1)
                } else {
                    None
                }
            })
            .unwrap_or(self.minimum_integer_register)
    }

    fn next_string_register(&self) -> u16 {
        self.instructions
            .iter()
            .rev()
            .find_map(|(instruction, r#type, _)| {
                if r#type == &Type::String {
                    Some(instruction.a_field() + 1)
                } else {
                    None
                }
            })
            .unwrap_or(self.minimum_string_register)
    }

    fn next_list_register(&mut self) -> u16 {
        self.instructions
            .iter()
            .rev()
            .find_map(|(instruction, r#type, _)| {
                if let Type::List { .. } = r#type {
                    Some(instruction.a_field() + 1)
                } else if instruction.operation() == Operation::LOAD_LIST {
                    Some(instruction.a_field() + 1)
                } else {
                    None
                }
            })
            .unwrap_or(self.minimum_list_register)
    }

    fn next_function_register(&self) -> u16 {
        self.instructions
            .iter()
            .rev()
            .find_map(|(instruction, _, _)| {
                if matches!(
                    instruction.operation(),
                    Operation::LOAD_FUNCTION | Operation::LOAD_SELF
                ) {
                    Some(instruction.a_field() + 1)
                } else {
                    None
                }
            })
            .unwrap_or(self.minimum_function_register)
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

    fn get_local(&self, index: u16) -> Result<&Local, CompileError> {
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
            .find_map(|(index, local)| {
                let identifier = self.string_constants.get(local.identifier_index as usize)?;

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

        let identifier = DustString::from(identifier);
        let identifier_index = self.push_or_get_constant_string(identifier);
        let local_index = self.locals.len() as u16;

        self.locals.push(Local::new(
            identifier_index,
            register_index,
            r#type,
            is_mutable,
            scope,
        ));

        (local_index, identifier_index)
    }

    fn get_identifier(&self, local_index: u16) -> Option<String> {
        self.locals.get(local_index as usize).and_then(|local| {
            self.string_constants
                .get(local.identifier_index as usize)
                .map(|value| value.to_string())
        })
    }

    fn push_or_get_constant_string(&mut self, string: DustString) -> u16 {
        if let Some(index) = self
            .string_constants
            .iter()
            .position(|constant| constant == &string)
        {
            index as u16
        } else {
            let index = self.string_constants.len() as u16;

            self.string_constants.push(string);

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

    /// Updates [`Self::type`] with the given [Type] as `return_type`.
    ///
    /// If [`Self::type`] is already set, it will check if the given [Type] is compatible.
    fn update_return_type(&mut self, new_return_type: Type) -> Result<(), CompileError> {
        if self.r#type.return_type.as_ref() != &Type::None {
            self.r#type
                .return_type
                .check(&new_return_type)
                .map_err(|conflict| CompileError::ReturnTypeConflict {
                    conflict,
                    position: self.previous_position,
                })?;
        }

        *self.r#type.return_type.as_mut() = new_return_type;

        Ok(())
    }

    fn emit_instruction(&mut self, instruction: Instruction, r#type: Type, position: Span) {
        debug!(
            "Emitting {} at {}",
            instruction.operation().to_string(),
            position.to_string()
        );

        self.instructions.push((instruction, r#type, position));
    }

    fn parse_boolean(&mut self) -> Result<(), CompileError> {
        let position = self.current_position;

        if let Token::Boolean(text) = self.current_token {
            self.advance()?;

            let boolean = text.parse::<bool>().unwrap() as u8;
            let destination = self.next_boolean_register();
            let load_encoded =
                Instruction::load_encoded(destination, boolean, TypeCode::BOOLEAN, false);

            self.emit_instruction(load_encoded, Type::Boolean, position);

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
            let destination = self.next_byte_register();
            let load_encoded = Instruction::load_encoded(destination, byte, TypeCode::BYTE, false);

            self.emit_instruction(load_encoded, Type::Byte, position);

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

            let destination = self.next_character_register();
            let constant_index = if let Some(index) = self
                .character_constants
                .iter()
                .position(|constant| constant == &character)
            {
                index as u16
            } else {
                let index = self.character_constants.len() as u16;

                self.character_constants.push(character);

                index
            };
            let load_constant =
                Instruction::load_constant(destination, constant_index, TypeCode::CHARACTER, false);

            self.emit_instruction(load_constant, Type::Character, position);

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
            let destination = self.next_float_register();
            let constant_index = if let Some(index) = self
                .float_constants
                .iter()
                .position(|constant| constant == &float)
            {
                index as u16
            } else {
                let index = self.float_constants.len() as u16;

                self.float_constants.push(float);

                index
            };
            let load_constant =
                Instruction::load_constant(destination, constant_index, TypeCode::FLOAT, false);

            self.emit_instruction(load_constant, Type::Float, position);

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

            let mut integer = 0_i64;

            for digit in text.chars() {
                let digit = if let Some(digit) = digit.to_digit(10) {
                    digit as i64
                } else {
                    continue;
                };

                integer = integer * 10 + digit;
            }

            let constant_index = if let Some(index) = self
                .integer_constants
                .iter()
                .position(|constant| constant == &integer)
            {
                index as u16
            } else {
                let index = self.integer_constants.len() as u16;

                self.integer_constants.push(integer);

                index
            };
            let destination = self.next_integer_register();
            let load_constant =
                Instruction::load_constant(destination, constant_index, TypeCode::INTEGER, false);

            self.emit_instruction(load_constant, Type::Integer, position);

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

            let string = DustString::from(text);
            let constant_index = if let Some(index) = self
                .string_constants
                .iter()
                .position(|constant| constant == &string)
            {
                index as u16
            } else {
                let index = self.string_constants.len() as u16;

                self.string_constants.push(string);

                index
            };
            let destination = self.next_string_register();
            let load_constant =
                Instruction::load_constant(destination, constant_index, TypeCode::STRING, false);

            self.emit_instruction(load_constant, Type::String, position);

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
        let (argument, push_back) = self.handle_binary_argument(&previous_instruction);

        if push_back {
            self.instructions.push((
                previous_instruction,
                previous_type.clone(),
                previous_position,
            ))
        }

        let (type_code, destination) = match previous_type {
            Type::Boolean => (TypeCode::BOOLEAN, self.next_boolean_register()),
            Type::Float => (TypeCode::FLOAT, self.next_float_register()),
            Type::Integer => (TypeCode::INTEGER, self.next_integer_register()),
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

    fn handle_binary_argument(&mut self, instruction: &Instruction) -> (Operand, bool) {
        let operand = instruction.as_operand();
        let push_back = match instruction.operation() {
            Operation::LOAD_ENCODED
            | Operation::LOAD_LIST
            | Operation::LOAD_SELF
            | Operation::CALL
            | Operation::CALL_NATIVE
            | Operation::ADD
            | Operation::SUBTRACT
            | Operation::MULTIPLY
            | Operation::DIVIDE
            | Operation::MODULO => true,
            _ => !instruction.yields_value(),
        };

        (operand, push_back)
    }

    fn parse_math_binary(&mut self) -> Result<(), CompileError> {
        let (left_instruction, left_type, left_position) =
            self.instructions
                .pop()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;
        let (left, push_back_left) = self.handle_binary_argument(&left_instruction);
        let left_is_mutable_local = if left_instruction.operation() == Operation::MOVE {
            let Move { operand: to, .. } = Move::from(&left_instruction);

            self.locals
                .get(to.index() as usize)
                .map(|local| local.is_mutable)
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

        if is_assignment && !left_is_mutable_local {
            return Err(CompileError::ExpectedMutableVariable {
                found: self.previous_token.to_owned(),
                position: left_position,
            });
        }

        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        if is_assignment {
            self.allow(Token::Semicolon)?;
        }

        let (right_instruction, right_type, right_position) = self.instructions.pop().unwrap();
        let (right, push_back_right) = self.handle_binary_argument(&right_instruction);

        check_math_type(&right_type, operator, &right_position)?;
        check_math_types(
            &left_type,
            &left_position,
            operator,
            &right_type,
            &right_position,
        )?;

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
            left.index()
        } else {
            match left_type {
                Type::Boolean => self.next_boolean_register(),
                Type::Byte => self.next_byte_register(),
                Type::Character => self.next_string_register(),
                Type::Float => self.next_float_register(),
                Type::Integer => self.next_integer_register(),
                Type::String => self.next_string_register(),
                _ => unreachable!(),
            }
        };
        let instruction = match operator {
            Token::Plus | Token::PlusEqual => Instruction::add(destination, left, right),
            Token::Minus | Token::MinusEqual => Instruction::subtract(destination, left, right),
            Token::Star | Token::StarEqual => Instruction::multiply(destination, left, right),
            Token::Slash | Token::SlashEqual => Instruction::divide(destination, left, right),
            Token::Percent | Token::PercentEqual => Instruction::modulo(destination, left, right),
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
        let position = Span(left_position.0, right_position.1);

        self.emit_instruction(instruction, r#type, position);

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

        // TODO: Check if the left type is a valid type for comparison

        let (left, push_back_left) = self.handle_binary_argument(&left_instruction);

        if push_back_left {
            self.instructions
                .push((left_instruction, left_type, left_position));
        }

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(&operator);

        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        let (right_instruction, right_type, right_position) =
            self.instructions
                .pop()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;

        // TODO: Check if the right type is a valid type for comparison

        let (right, push_back_right) = self.handle_binary_argument(&right_instruction);

        if push_back_right {
            self.instructions
                .push((right_instruction, right_type, right_position));
        }

        // TODO: Check if the left and right types are compatible

        let comparison = match operator {
            Token::DoubleEqual => Instruction::equal(true, left, right),
            Token::BangEqual => Instruction::equal(false, left, right),
            Token::Less => Instruction::less(true, left, right),
            Token::LessEqual => Instruction::less_equal(true, left, right),
            Token::Greater => Instruction::less_equal(false, left, right),
            Token::GreaterEqual => Instruction::less(false, left, right),
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
        let destination = self.next_boolean_register();
        let load_true = Instruction::load_encoded(destination, true as u8, TypeCode::BOOLEAN, true);
        let load_false =
            Instruction::load_encoded(destination, false as u8, TypeCode::BOOLEAN, false);
        let comparison_position = Span(left_position.0, right_position.1);

        self.emit_instruction(comparison, Type::Boolean, comparison_position);
        self.emit_instruction(jump, Type::None, comparison_position);
        self.emit_instruction(load_true, Type::Boolean, comparison_position);
        self.emit_instruction(load_false, Type::Boolean, comparison_position);

        Ok(())
    }

    fn parse_logical_binary(&mut self) -> Result<(), CompileError> {
        let (left_instruction, left_type, left_position) =
            self.instructions
                .pop()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;

        // TODO: Check if the left type is boolean

        let (left, push_back_left) = self.handle_binary_argument(&left_instruction);

        if push_back_left {
            self.instructions
                .push((left_instruction, left_type.clone(), left_position));
        }

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
        let test = Instruction::test(left.index(), test_boolean);
        let jump = Instruction::jump(1, true);

        self.emit_instruction(test, Type::None, operator_position);
        self.emit_instruction(jump, Type::None, operator_position);

        let instruction_count_before_right = self.instructions.len();

        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        // TODO: Check if the right type is boolean

        let instruction_count = self.instructions.len();

        if instruction_count == instruction_count_before_right + 1 {
            self.instructions
                .last_mut()
                .unwrap()
                .0
                .set_a_field(left.index());
        } else if matches!(
            self.get_last_operations(),
            Some([
                Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL,
                Operation::JUMP,
                Operation::LOAD_ENCODED | Operation::LOAD_CONSTANT,
                Operation::LOAD_ENCODED | Operation::LOAD_CONSTANT,
            ])
        ) {
            let loaders = if cfg!(debug_assertions) {
                self.instructions
                    .get_disjoint_mut([instruction_count - 1, instruction_count - 2])
                    .unwrap() // Safe because the indices in bounds and do not overlap
            } else {
                unsafe {
                    self.instructions
                        .get_disjoint_unchecked_mut([instruction_count - 1, instruction_count - 2])
                }
            };

            loaders[0].0.set_a_field(left.index());
            loaders[1].0.set_a_field(left.index());
        }

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
            return self.parse_call_native(native_function, start_position);
        } else if self.function_name.as_deref() == Some(identifier) && !self.is_main {
            let destination = self.next_function_register();
            let load_self = Instruction::load_self(destination, false);

            self.emit_instruction(load_self, Type::SelfFunction, start_position);

            return Ok(());
        } else {
            return Err(CompileError::UndeclaredVariable {
                identifier: identifier.to_string(),
                position: start_position,
            });
        };

        let local = self.get_local(local_index)?;
        let local_type = local.r#type.clone();
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
            }
        }

        let destination = match local_type {
            Type::Boolean => self.next_boolean_register(),
            Type::Byte => self.next_byte_register(),
            Type::Character => self.next_character_register(),
            Type::Float => self.next_float_register(),
            Type::Integer => self.next_integer_register(),
            Type::String => self.next_string_register(),
            Type::List(_) => self.next_list_register(),
            Type::Function(_) => self.next_function_register(),
            _ => todo!(),
        };
        let point = Instruction::r#move(
            destination,
            Operand::Register(local_register_index, local_type.type_code()),
        );

        self.emit_instruction(point, local_type, self.previous_position);

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

        let mut item_type = Type::None;
        let mut start_register = None;

        while !self.allow(Token::RightBracket)? {
            let next_boolean_register = self.next_boolean_register();
            let next_byte_register = self.next_byte_register();
            let next_character_register = self.next_character_register();
            let next_float_register = self.next_float_register();
            let next_integer_register = self.next_integer_register();
            let next_string_register = self.next_string_register();
            let next_list_register = self.next_list_register();

            self.parse_expression()?;
            self.allow(Token::Comma)?;

            if item_type == Type::None {
                item_type = self.get_last_instruction_type();
            } else {
                // TODO: Check if the item type the same as the previous item type
            }

            if start_register.is_none() {
                let first_index = self.instructions.last().unwrap().0.a_field();

                start_register = Some(first_index);
            }

            match self.get_last_instruction_type() {
                Type::Boolean => {
                    let used_boolean_registers =
                        self.next_boolean_register() - next_boolean_register;

                    if used_boolean_registers > 1 {
                        let close = Instruction::close(
                            next_boolean_register,
                            self.next_boolean_register() - 2,
                            TypeCode::BOOLEAN,
                        );

                        self.emit_instruction(close, Type::None, self.current_position);
                    }
                }
                Type::Byte => {
                    let used_byte_registers = self.next_byte_register() - next_byte_register;

                    if used_byte_registers > 1 {
                        let close = Instruction::close(
                            next_byte_register,
                            self.next_byte_register() - 2,
                            TypeCode::BYTE,
                        );

                        self.emit_instruction(close, Type::None, self.current_position);
                    }
                }
                Type::Character => {
                    let used_character_registers =
                        self.next_character_register() - next_character_register;

                    if used_character_registers > 1 {
                        let close = Instruction::close(
                            next_character_register,
                            self.next_character_register() - 2,
                            TypeCode::CHARACTER,
                        );

                        self.emit_instruction(close, Type::None, self.current_position);
                    }
                }
                Type::Float => {
                    let used_float_registers = self.next_float_register() - next_float_register;

                    if used_float_registers > 1 {
                        let close = Instruction::close(
                            next_float_register,
                            self.next_float_register() - 2,
                            TypeCode::FLOAT,
                        );

                        self.emit_instruction(close, Type::None, self.current_position);
                    }
                }
                Type::Integer => {
                    let used_integer_registers =
                        self.next_integer_register() - next_integer_register;

                    if used_integer_registers > 1 {
                        let close = Instruction::close(
                            next_integer_register,
                            self.next_integer_register() - 2,
                            TypeCode::INTEGER,
                        );

                        self.emit_instruction(close, Type::None, self.current_position);
                    }
                }
                Type::String => {
                    let used_string_registers = self.next_string_register() - next_string_register;

                    if used_string_registers > 1 {
                        let close = Instruction::close(
                            next_string_register,
                            self.next_string_register() - 2,
                            TypeCode::STRING,
                        );

                        self.emit_instruction(close, Type::None, self.current_position);
                    }
                }
                Type::List { .. } => {
                    let used_list_registers = self.next_list_register() - next_list_register;

                    if used_list_registers > 1 {
                        let close = Instruction::close(
                            next_list_register,
                            self.next_list_register() - 2,
                            TypeCode::LIST,
                        );

                        self.emit_instruction(close, Type::None, self.current_position);
                    }
                }
                _ => unimplemented!(),
            };
        }

        let end = self.previous_position.1;
        let end_register = match item_type {
            Type::Boolean => self.next_boolean_register().saturating_sub(1),
            Type::Byte => self.next_byte_register().saturating_sub(1),
            Type::Character => self.next_character_register().saturating_sub(1),
            Type::Float => self.next_float_register().saturating_sub(1),
            Type::Integer => self.next_integer_register().saturating_sub(1),
            Type::String => self.next_string_register().saturating_sub(1),
            Type::List { .. } => self.next_list_register().saturating_sub(1),
            _ => todo!(),
        };
        let destination = self.next_list_register();
        let load_list = Instruction::load_list(
            destination,
            item_type.type_code(),
            start_register.unwrap_or(0),
            end_register,
            false,
        );
        let list_length = end_register - start_register.unwrap_or(0) + 1;

        if list_length == 1 && self.get_last_operation() == Some(Operation::CLOSE) {
            self.instructions.pop();
        }

        self.emit_instruction(
            load_list,
            Type::List(item_type.type_code()),
            Span(start, end),
        );

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
                Operation::LOAD_ENCODED,
                Operation::LOAD_ENCODED
            ]),
        ) {
            self.instructions.pop();
            self.instructions.pop();
            self.instructions.pop();
        } else {
            let operand_register = match self.get_last_instruction_type() {
                Type::Boolean => self.next_boolean_register() - 1,
                Type::Byte => self.next_byte_register() - 1,
                Type::Character => self.next_character_register() - 1,
                Type::Float => self.next_float_register() - 1,
                Type::Integer => self.next_integer_register() - 1,
                Type::String => self.next_string_register() - 1,
                _ => todo!(),
            };
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
        let mut if_block_distance = if_block_end - if_block_start;

        let (if_block_last_instruction, if_block_type, _) = self.instructions.last().unwrap();
        let if_block_type = if_block_type.clone();
        let if_block_last_instruction_destination = if_block_last_instruction.a_field();

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
        let else_block_distance = else_block_end - if_block_end;
        let (else_block_last_instruction, else_block_type, _) =
            self.instructions.last_mut().unwrap();

        else_block_last_instruction.set_a_field(if_block_last_instruction_destination);

        if let Err(conflict) = if_block_type.check(else_block_type) {
            return Err(CompileError::IfElseBranchMismatch {
                conflict,
                position: Span(if_block_start_position.0, self.current_position.1),
            });
        }

        match else_block_distance {
            0 => {}
            1 => {
                if let Some([Operation::LOAD_ENCODED | Operation::LOAD_CONSTANT, _]) =
                    self.get_last_operations()
                {
                    let loader_index = self.instructions.len() - 2;
                    let (loader, _, _) = self.instructions.get_mut(loader_index).unwrap();

                    loader.set_c_field(true as u16);
                } else {
                    if_block_distance += 1;
                    let jump = Instruction::from(Jump {
                        offset: else_block_distance as u16,
                        is_positive: true,
                    });

                    self.instructions
                        .insert(if_block_end, (jump, Type::None, self.current_position));
                }
            }
            2.. => {
                if_block_distance += 1;
                let jump = Instruction::from(Jump {
                    offset: else_block_distance as u16,
                    is_positive: true,
                });

                self.instructions
                    .insert(if_block_end, (jump, Type::None, self.current_position));
            }
        }

        let jump = Instruction::jump(if_block_distance as u16, true);

        self.instructions
            .insert(if_block_start, (jump, Type::None, if_block_start_position));

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
                Operation::LOAD_ENCODED,
                Operation::LOAD_ENCODED
            ]),
        ) {
            self.instructions.pop();
            self.instructions.pop();
            self.instructions.pop();
        } else {
            let operand_register = match self.get_last_instruction_type() {
                Type::Boolean => self.next_boolean_register() - 1,
                Type::Byte => self.next_byte_register() - 1,
                Type::Character => self.next_character_register() - 1,
                Type::Float => self.next_float_register() - 1,
                Type::Integer => self.next_integer_register() - 1,
                Type::String => self.next_string_register() - 1,
                _ => todo!(),
            };
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

        let (should_return_value, type_code, return_register) =
            if matches!(self.current_token, Token::Semicolon | Token::RightBrace) {
                self.update_return_type(Type::None)?;

                (false, TypeCode::NONE, 0)
            } else {
                self.parse_expression()?;

                let expression_type = self.get_last_instruction_type();
                let type_code = expression_type.type_code();
                let return_register = match expression_type {
                    Type::Boolean => self.next_boolean_register() - 1,
                    Type::Byte => self.next_byte_register() - 1,
                    Type::Character => self.next_character_register() - 1,
                    Type::Float => self.next_float_register() - 1,
                    Type::Integer => self.next_integer_register() - 1,
                    Type::String => self.next_string_register() - 1,
                    _ => todo!(),
                };

                self.update_return_type(expression_type)?;

                (true, type_code, return_register)
            };
        let end = self.current_position.1;
        let r#return = Instruction::from(Return {
            should_return_value,
            return_register,
            r#type: type_code,
        });

        self.emit_instruction(r#return, Type::None, Span(start, end));

        let instruction_length = self.instructions.len();

        for (index, (instruction, _, _)) in self.instructions.iter_mut().enumerate() {
            if instruction.operation() == Operation::JUMP {
                let Jump {
                    offset,
                    is_positive,
                } = Jump::from(&*instruction);
                let offset = offset as usize;

                if is_positive && offset + index == instruction_length - 1 {
                    *instruction = Instruction::jump((offset + 1) as u16, true);
                }
            }
        }

        Ok(())
    }

    fn parse_implicit_return(&mut self) -> Result<(), CompileError> {
        if matches!(self.get_last_operation(), Some(Operation::MOVE)) {
            let last_instruction = self.instructions.last().unwrap().0;
            let Move { operand, .. } = Move::from(&last_instruction);

            let (r#move, r#type, position) = self.instructions.pop().unwrap();
            let (should_return, target_register) = if r#type == Type::None {
                (false, 0)
            } else {
                (true, operand.index())
            };
            let r#return =
                Instruction::r#return(should_return, target_register, r#type.type_code());

            if !should_return {
                self.instructions.push((r#move, r#type.clone(), position));
            }

            self.emit_instruction(r#return, r#type, self.current_position);
        } else if matches!(self.get_last_operation(), Some(Operation::RETURN))
            || matches!(
                self.get_last_operations(),
                Some([Operation::RETURN, Operation::JUMP])
            )
        {
            // Do nothing if the last instruction is a return or a return followed by a jump
        } else if self.allow(Token::Semicolon)? {
            let r#return = Instruction::r#return(false, 0, TypeCode::NONE);

            self.emit_instruction(r#return, Type::None, self.current_position);
        } else if let Some((mut previous_expression_type, previous_destination_register)) =
            self.instructions.last().map(|(instruction, r#type, _)| {
                if r#type == &Type::None {
                    (Type::None, 0)
                } else if instruction.yields_value() {
                    (r#type.clone(), instruction.a_field())
                } else {
                    (Type::None, 0)
                }
            })
        {
            if let Type::Function(_) = previous_expression_type {
                if let Some((instruction, _, _)) = self.instructions.last() {
                    let LoadFunction {
                        prototype_index, ..
                    } = LoadFunction::from(*instruction);

                    let function_type = self
                        .prototypes
                        .get(prototype_index as usize)
                        .map(|prototype| Type::Function(prototype.r#type.clone()))
                        .unwrap_or(Type::None);

                    previous_expression_type = function_type;
                }
            }

            let should_return_value = previous_expression_type != Type::None;
            let r#return = Instruction::r#return(
                should_return_value,
                previous_destination_register,
                previous_expression_type.type_code(),
            );

            self.update_return_type(previous_expression_type.clone())?;
            self.emit_instruction(r#return, Type::None, self.current_position);
        }

        let instruction_length = self.instructions.len();

        for (index, (instruction, _, _)) in self.instructions.iter_mut().enumerate() {
            if instruction.operation() == Operation::JUMP {
                let Jump {
                    offset,
                    is_positive,
                } = Jump::from(&*instruction);
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
        self.allow(Token::Semicolon)?;

        let register_index = match self.get_last_instruction_type() {
            Type::Boolean => self.next_boolean_register() - 1,
            Type::Byte => self.next_byte_register() - 1,
            Type::Character => self.next_character_register() - 1,
            Type::Float => self.next_float_register() - 1,
            Type::Integer => self.next_integer_register() - 1,
            Type::String => self.next_string_register() - 1,
            _ => todo!(),
        };
        let r#type = if let Some(r#type) = explicit_type {
            r#type
        } else {
            self.get_last_instruction_type()
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

        let mut value_parameters = Vec::with_capacity(3);

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

            let local_register_index = match r#type {
                Type::Boolean => function_compiler.next_boolean_register(),
                Type::Byte => function_compiler.next_byte_register(),
                Type::Character => function_compiler.next_character_register(),
                Type::Float => function_compiler.next_float_register(),
                Type::Integer => function_compiler.next_integer_register(),
                Type::String => function_compiler.next_string_register(),
                _ => todo!(),
            };
            function_compiler.declare_local(
                parameter,
                local_register_index,
                r#type.clone(),
                is_mutable,
                function_compiler.current_scope,
            );

            match r#type {
                Type::Boolean => function_compiler.minimum_boolean_register += 1,
                Type::Byte => function_compiler.minimum_byte_register += 1,
                Type::Character => function_compiler.minimum_character_register += 1,
                Type::Float => function_compiler.minimum_float_register += 1,
                Type::Integer => function_compiler.minimum_integer_register += 1,
                Type::String => function_compiler.minimum_string_register += 1,
                _ => {}
            }

            value_parameters.push(r#type);
            function_compiler.allow(Token::Comma)?;
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
        function_compiler.r#type = FunctionType::new([], value_parameters, return_type);

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
        let destination = self.next_function_register();
        let load_function = Instruction::load_function(destination, prototype_index, false);
        let r#type = Type::Function(chunk.r#type.clone());

        if let Some(identifier) = identifier {
            self.declare_local(
                identifier,
                destination,
                Type::Function(chunk.r#type.clone()),
                false,
                self.current_scope,
            );
        }

        self.prototypes.push(Arc::new(chunk));
        self.emit_instruction(load_function, r#type, Span(function_start, function_end));

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
        let b_field = last_instruction.b_field();

        let function_return_type = match last_instruction_type {
            Type::Function(function_type) => *function_type.return_type.clone(),
            Type::SelfFunction => *self.r#type.return_type.clone(),
            _ => {
                return Err(CompileError::ExpectedFunction {
                    found: self.previous_token.to_owned(),
                    actual_type: last_instruction_type.clone(),
                    position: self.previous_position,
                });
            }
        };
        let last_operation = last_instruction.operation();
        let function_register = if matches!(
            last_operation,
            Operation::LOAD_FUNCTION | Operation::LOAD_SELF
        ) {
            last_instruction.a_field()
        } else if last_instruction.operation() == Operation::MOVE {
            self.instructions.pop();

            b_field
        } else {
            return Err(CompileError::ExpectedFunction {
                found: self.previous_token.to_owned(),
                actual_type: last_instruction_type.clone(),
                position: self.previous_position,
            });
        };

        let type_argument_list = Vec::new();

        let mut value_argument_list = Vec::new();

        while !self.allow(Token::RightParenthesis)? {
            self.parse_expression()?;
            self.allow(Token::Comma)?;

            let argument_index = match self.get_last_instruction_type() {
                Type::Boolean => self.next_boolean_register() - 1,
                Type::Byte => self.next_byte_register() - 1,
                Type::Character => self.next_character_register() - 1,
                Type::Float => self.next_float_register() - 1,
                Type::Integer => self.next_integer_register() - 1,
                Type::String => self.next_string_register() - 1,
                _ => todo!(),
            };

            value_argument_list.push(argument_index);
        }

        let argument_list_index = self.argument_lists.len() as u16;

        self.argument_lists
            .push((value_argument_list, type_argument_list));

        let end = self.current_position.1;
        let destination = match function_return_type {
            Type::None => 0,
            Type::Boolean => self.next_boolean_register(),
            Type::Byte => self.next_byte_register(),
            Type::Character => self.next_character_register(),
            Type::Float => self.next_float_register(),
            Type::Integer => self.next_integer_register(),
            Type::String => self.next_string_register(),
            _ => todo!(),
        };
        let is_recursive = last_operation == Operation::LOAD_SELF;
        let call = Instruction::call(
            destination,
            function_register,
            argument_list_index,
            function_return_type.type_code(),
            is_recursive,
        );

        self.emit_instruction(call, function_return_type, Span(start, end));

        Ok(())
    }

    fn parse_call_native(
        &mut self,
        function: NativeFunction,
        start: Span,
    ) -> Result<(), CompileError> {
        let mut type_argument_list = Vec::new();

        if self.allow(Token::Less)? {
            while !self.allow(Token::Greater)? {
                let r#type = self.parse_type_from(self.current_token, self.current_position)?;

                type_argument_list.push(r#type);
                self.advance()?;

                self.allow(Token::Comma)?;
            }
        }

        self.expect(Token::LeftParenthesis)?;

        let mut value_argument_list = Vec::new();

        while !self.allow(Token::RightParenthesis)? {
            self.parse_expression()?;
            self.allow(Token::Comma)?;

            let argument_index = match self.get_last_instruction_type() {
                Type::Boolean => self.next_boolean_register() - 1,
                Type::Byte => self.next_byte_register() - 1,
                Type::Character => self.next_character_register() - 1,
                Type::Float => self.next_float_register() - 1,
                Type::Integer => self.next_integer_register() - 1,
                Type::String => self.next_string_register() - 1,
                _ => todo!(),
            };

            value_argument_list.push(argument_index);
        }

        let argument_list_index = self.argument_lists.len() as u16;

        self.argument_lists
            .push((value_argument_list, type_argument_list));

        let end = self.current_position.1;
        let return_type = function.r#type().return_type.as_ref().clone();
        let destination = match return_type {
            Type::None => 0,
            Type::Boolean => self.next_boolean_register(),
            Type::Byte => self.next_byte_register(),
            Type::Character => self.next_character_register(),
            Type::Float => self.next_float_register(),
            Type::Integer => self.next_integer_register(),
            Type::String => self.next_string_register(),
            _ => todo!(),
        };
        let call_native = Instruction::call_native(destination, function, argument_list_index);

        self.emit_instruction(call_native, return_type, Span(start.0, end));

        Ok(())
    }

    fn parse_semicolon(&mut self) -> Result<(), CompileError> {
        let (_, last_instruction_type, _) = self.instructions.last_mut().unwrap();

        *last_instruction_type = Type::None;

        self.advance()?;

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
                self.current_token,
            );

            prefix_parser(self)?;
        }

        let mut infix_rule = ParseRule::from(&self.current_token);

        while precedence <= infix_rule.precedence {
            if let Some(infix_parser) = infix_rule.infix {
                debug!(
                    "{} is infix with precedence {precedence}",
                    self.current_token,
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
