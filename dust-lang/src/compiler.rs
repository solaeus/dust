//! Compilation tools and errors
//!
//! This module provides two compilation options:
//! - [`compile`], which compiles the entire input and returns a chunk
//! - [`Compiler`], which compiles the input a token at a time while assembling a chunk
use std::{
    fmt::{self, Display, Formatter},
    mem::replace,
    num::{ParseFloatError, ParseIntError},
    vec,
};

use colored::Colorize;

use crate::{
    value::ConcreteValue, AnnotatedError, Chunk, ChunkError, DustError, FunctionType, Instruction,
    LexError, Lexer, Local, NativeFunction, Operation, Optimizer, Scope, Span, Token, TokenKind,
    TokenOwned, Type, TypeConflict,
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
/// assert_eq!(chunk.len(), 6);
/// ```
pub fn compile(source: &str) -> Result<Chunk, DustError> {
    let lexer = Lexer::new(source);
    let mut compiler =
        Compiler::new(lexer).map_err(|error| DustError::Compile { error, source })?;

    compiler
        .parse_top_level()
        .map_err(|error| DustError::Compile { error, source })?;

    Ok(compiler.finish())
}

/// Low-level tool for compiling the input a token at a time while assembling a chunk.
///
/// See the [`compile`] function an example of how to create and use a Compiler.
#[derive(Debug, Eq, PartialEq, PartialOrd)]
pub struct Compiler<'src> {
    chunk: Chunk,
    lexer: Lexer<'src>,

    local_definitions: Vec<u8>,
    optimization_count: usize,
    previous_expression_type: Type,
    minimum_register: u8,

    current_token: Token<'src>,
    current_position: Span,

    previous_token: Token<'src>,
    previous_position: Span,

    block_index: u8,
    current_scope: Scope,
}

impl<'src> Compiler<'src> {
    pub fn new(mut lexer: Lexer<'src>) -> Result<Self, CompileError> {
        let (current_token, current_position) = lexer.next_token()?;
        let chunk = Chunk::new(None);

        log::info!(
            "Begin chunk with {} at {}",
            current_token.to_string().bold(),
            current_position.to_string()
        );

        Ok(Compiler {
            chunk,
            lexer,
            local_definitions: Vec::new(),
            optimization_count: 0,
            previous_expression_type: Type::None,
            minimum_register: 0,
            current_token,
            current_position,
            previous_token: Token::Eof,
            previous_position: Span(0, 0),
            block_index: 0,
            current_scope: Scope::default(),
        })
    }

    pub fn finish(self) -> Chunk {
        log::info!("End chunk with {} optimizations", self.optimization_count);

        self.chunk
    }

    fn is_eof(&self) -> bool {
        matches!(self.current_token, Token::Eof)
    }

    fn next_register(&mut self) -> u8 {
        self.chunk
            .instructions()
            .iter()
            .rev()
            .find_map(|(instruction, _)| {
                if instruction.yields_value() {
                    let previous = instruction.a();
                    let next = previous.overflowing_add(1).0;

                    Some(next)
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

        log::info!(
            "Parsing {} at {}",
            new_token.to_string().bold(),
            position.to_string()
        );

        self.previous_token = replace(&mut self.current_token, new_token);
        self.previous_position = replace(&mut self.current_position, position);

        Ok(())
    }

    fn get_local(&self, index: u8) -> Result<&Local, CompileError> {
        self.chunk
            .get_local(index)
            .map_err(|error| CompileError::Chunk {
                error,
                position: self.current_position,
            })
    }

    fn get_local_index(&self, identifier_text: &str) -> Result<u8, CompileError> {
        self.chunk
            .locals()
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, local)| {
                let constant = self
                    .chunk
                    .constants()
                    .get(local.identifier_index as usize)?;
                let identifier = if let ConcreteValue::String(identifier) = constant {
                    identifier
                } else {
                    return None;
                };

                if identifier == identifier_text {
                    Some(index as u8)
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
        r#type: Type,
        is_mutable: bool,
        scope: Scope,
        register_index: u8,
    ) -> (u8, u8) {
        log::debug!("Declare local {identifier}");

        let identifier = ConcreteValue::string(identifier);
        let identifier_index = self.chunk.push_or_get_constant(identifier);

        self.chunk
            .locals_mut()
            .push(Local::new(identifier_index, r#type, is_mutable, scope));
        self.local_definitions.push(register_index);

        (self.chunk.locals().len() as u8 - 1, identifier_index)
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

    fn pop_last_instruction(&mut self) -> Result<(Instruction, Span), CompileError> {
        self.chunk
            .instructions_mut()
            .pop()
            .ok_or_else(|| CompileError::ExpectedExpression {
                found: self.previous_token.to_owned(),
                position: self.previous_position,
            })
    }

    fn get_last_operations<const COUNT: usize>(&self) -> Option<[Operation; COUNT]> {
        let mut n_operations = [Operation::Return; COUNT];

        for (nth, operation) in n_operations.iter_mut().rev().zip(
            self.chunk
                .instructions()
                .iter()
                .rev()
                .map(|(instruction, _)| instruction.operation()),
        ) {
            *nth = operation;
        }

        Some(n_operations)
    }

    fn get_last_jumpable_mut(&mut self) -> Option<&mut Instruction> {
        self.chunk
            .instructions_mut()
            .iter_mut()
            .rev()
            .find_map(|(instruction, _)| {
                if let Operation::LoadBoolean | Operation::LoadConstant = instruction.operation() {
                    Some(instruction)
                } else {
                    None
                }
            })
    }

    pub fn get_instruction_type(&self, instruction: &Instruction) -> Result<Type, CompileError> {
        use Operation::*;

        match instruction.operation() {
            Add | Divide | Modulo | Multiply | Subtract => {
                if instruction.b_is_constant() {
                    self.chunk
                        .get_constant_type(instruction.b())
                        .map_err(|error| CompileError::Chunk {
                            error,
                            position: self.current_position,
                        })
                } else {
                    self.get_register_type(instruction.b())
                }
            }
            LoadBoolean | Not => Ok(Type::Boolean),
            Negate => {
                if instruction.b_is_constant() {
                    self.chunk
                        .get_constant_type(instruction.b())
                        .map_err(|error| CompileError::Chunk {
                            error,
                            position: self.current_position,
                        })
                } else {
                    self.get_register_type(instruction.b())
                }
            }
            LoadConstant => self
                .chunk
                .get_constant_type(instruction.b())
                .map_err(|error| CompileError::Chunk {
                    error,
                    position: self.current_position,
                }),
            LoadList => self.get_register_type(instruction.a()),
            GetLocal => self
                .chunk
                .get_local_type(instruction.b())
                .cloned()
                .map_err(|error| CompileError::Chunk {
                    error,
                    position: self.current_position,
                }),
            CallNative => {
                let native_function = NativeFunction::from(instruction.b());

                Ok(*native_function.r#type().return_type)
            }
            _ => Ok(Type::None),
        }
    }

    pub fn get_register_type(&self, register_index: u8) -> Result<Type, CompileError> {
        for (index, (instruction, _)) in self.chunk.instructions().iter().enumerate() {
            if instruction.a() == register_index {
                if let Operation::LoadList = instruction.operation() {
                    let mut length = (instruction.c() - instruction.b() + 1) as usize;
                    let mut item_type = Type::Any;
                    let distance_to_end = self.chunk.len() - index;

                    for (instruction, _) in self
                        .chunk
                        .instructions()
                        .iter()
                        .rev()
                        .skip(distance_to_end)
                        .take(length)
                    {
                        if let Operation::Close = instruction.operation() {
                            length -= (instruction.c() - instruction.b()) as usize;
                        } else if let Type::Any = item_type {
                            item_type = self.get_instruction_type(instruction)?;
                        }
                    }

                    return Ok(Type::List {
                        item_type: Box::new(item_type),
                        length,
                    });
                }

                if let Operation::LoadSelf = instruction.operation() {
                    return Ok(Type::SelfChunk);
                }

                if instruction.yields_value() {
                    return self.get_instruction_type(instruction);
                }
            }
        }

        Err(CompileError::CannotResolveRegisterType {
            register_index: register_index as usize,
            position: self.current_position,
        })
    }

    fn emit_instruction(&mut self, instruction: Instruction, position: Span) {
        log::debug!(
            "Emitting {} at {}",
            instruction.operation().to_string().bold(),
            position.to_string()
        );

        self.chunk.instructions_mut().push((instruction, position));
    }

    fn emit_constant(
        &mut self,
        constant: ConcreteValue,
        position: Span,
    ) -> Result<(), CompileError> {
        let constant_index = self.chunk.push_or_get_constant(constant);
        let register = self.next_register();

        self.emit_instruction(
            Instruction::load_constant(register, constant_index, false),
            position,
        );

        Ok(())
    }

    fn parse_boolean(&mut self) -> Result<(), CompileError> {
        let position = self.current_position;

        if let Token::Boolean(text) = self.current_token {
            self.advance()?;

            let boolean = text.parse::<bool>().unwrap();
            let register = self.next_register();

            self.emit_instruction(
                Instruction::load_boolean(register, boolean, false),
                position,
            );

            self.previous_expression_type = Type::Boolean;

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

            self.previous_expression_type = Type::Byte;

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

            self.previous_expression_type = Type::Character;

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

            self.previous_expression_type = Type::Float;

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

            let integer = text
                .parse::<i64>()
                .map_err(|error| CompileError::ParseIntError {
                    error,
                    position: self.previous_position,
                })?;
            let value = ConcreteValue::Integer(integer);

            self.emit_constant(value, position)?;

            self.previous_expression_type = Type::Integer;

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

            self.previous_expression_type = Type::String {
                length: Some(text.len()),
            };

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

        let (previous_instruction, previous_position) = self.pop_last_instruction()?;
        let (push_back, is_constant, argument) = {
            match previous_instruction.operation() {
                Operation::GetLocal => (false, false, previous_instruction.a()),
                Operation::LoadConstant => (false, true, previous_instruction.a()),
                Operation::LoadBoolean => (true, false, previous_instruction.a()),
                Operation::Close => {
                    return Err(CompileError::ExpectedExpression {
                        found: self.previous_token.to_owned(),
                        position: self.previous_position,
                    });
                }
                _ => (true, false, previous_instruction.a()),
            }
        };

        if push_back {
            self.emit_instruction(previous_instruction, previous_position);
        }

        let register = self.next_register();
        let mut instruction = match operator.kind() {
            TokenKind::Bang => Instruction::not(register, argument),
            TokenKind::Minus => Instruction::negate(register, argument),
            _ => {
                return Err(CompileError::ExpectedTokenMultiple {
                    expected: &[TokenKind::Bang, TokenKind::Minus],
                    found: operator.to_owned(),
                    position: operator_position,
                })
            }
        };

        if is_constant {
            instruction.set_b_is_constant();
        }

        self.emit_instruction(instruction, operator_position);

        if let TokenKind::Bang = operator.kind() {
            self.previous_expression_type = Type::Boolean;
        }

        Ok(())
    }

    fn handle_binary_argument(
        &mut self,
        instruction: &Instruction,
    ) -> Result<(bool, bool, bool, u8), CompileError> {
        let mut push_back = false;
        let mut is_constant = false;
        let mut is_mutable_local = false;
        let argument = match instruction.operation() {
            Operation::GetLocal => {
                let local_index = instruction.b();
                let local = self.get_local(local_index)?;
                is_mutable_local = local.is_mutable;

                *self
                    .local_definitions
                    .get(local_index as usize)
                    .ok_or_else(|| {
                        let identifier = self
                            .chunk
                            .constants()
                            .get(local.identifier_index as usize)
                            .unwrap()
                            .to_string();

                        CompileError::UndeclaredVariable {
                            identifier,
                            position: self.current_position,
                        }
                    })?
            }
            Operation::LoadConstant => {
                is_constant = true;

                instruction.b()
            }
            Operation::Close => {
                return Err(CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                });
            }
            _ => {
                push_back = true;

                if instruction.yields_value() {
                    instruction.a()
                } else {
                    self.next_register()
                }
            }
        };

        Ok((push_back, is_constant, is_mutable_local, argument))
    }

    fn parse_math_binary(&mut self) -> Result<(), CompileError> {
        let (left_instruction, left_position) =
            self.chunk.instructions_mut().pop().ok_or_else(|| {
                CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                }
            })?;
        let (push_back_left, left_is_constant, left_is_mutable_local, left) =
            self.handle_binary_argument(&left_instruction)?;

        if push_back_left {
            self.emit_instruction(left_instruction, left_position);
        }

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(&operator);

        if let Token::PlusEqual | Token::MinusEqual | Token::StarEqual | Token::SlashEqual =
            operator
        {
            if !left_is_mutable_local {
                return Err(CompileError::ExpectedMutableVariable {
                    found: self.previous_token.to_owned(),
                    position: left_position,
                });
            }
        }

        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        let (right_instruction, right_position) = self.pop_last_instruction()?;
        let (push_back_right, right_is_constant, _, right) =
            self.handle_binary_argument(&right_instruction)?;

        if push_back_right {
            self.emit_instruction(right_instruction, right_position);
        }

        let register = if left_is_mutable_local {
            left
        } else {
            self.next_register()
        };

        let mut new_instruction = match operator {
            Token::Plus => Instruction::add(register, left, right),
            Token::PlusEqual => Instruction::add(register, left, right),
            Token::Minus => Instruction::subtract(register, left, right),
            Token::MinusEqual => Instruction::subtract(register, left, right),
            Token::Star => Instruction::multiply(register, left, right),
            Token::StarEqual => Instruction::multiply(register, left, right),
            Token::Slash => Instruction::divide(register, left, right),
            Token::SlashEqual => Instruction::divide(register, left, right),
            Token::Percent => Instruction::modulo(register, left, right),
            Token::PercentEqual => Instruction::modulo(register, left, right),
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
                })
            }
        };

        if left_is_constant {
            new_instruction.set_b_is_constant();
        }

        if right_is_constant {
            new_instruction.set_c_is_constant();
        }

        self.emit_instruction(new_instruction, operator_position);

        if let Token::PlusEqual
        | Token::MinusEqual
        | Token::StarEqual
        | Token::SlashEqual
        | Token::PercentEqual = operator
        {
            self.previous_expression_type = Type::None;
        } else {
            self.previous_expression_type = self.get_instruction_type(&left_instruction)?;
        }

        Ok(())
    }

    fn parse_comparison_binary(&mut self) -> Result<(), CompileError> {
        if let Some([Operation::Equal | Operation::Less | Operation::LessEqual, _, _, _]) =
            self.get_last_operations()
        {
            return Err(CompileError::CannotChainComparison {
                position: self.current_position,
            });
        }

        let (left_instruction, left_position) =
            self.chunk.instructions_mut().pop().ok_or_else(|| {
                CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                }
            })?;
        let (push_back_left, left_is_constant, _, left) =
            self.handle_binary_argument(&left_instruction)?;

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(&operator);

        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        let (right_instruction, right_position) =
            self.chunk.instructions_mut().pop().ok_or_else(|| {
                CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                }
            })?;
        let (push_back_right, right_is_constant, _, right) =
            self.handle_binary_argument(&right_instruction)?;

        let mut instruction = match operator {
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
                })
            }
        };

        if left_is_constant {
            instruction.set_b_is_constant();
        }

        if right_is_constant {
            instruction.set_c_is_constant();
        }

        if push_back_left {
            self.emit_instruction(left_instruction, left_position);
        }

        if push_back_right {
            self.emit_instruction(right_instruction, right_position);
        }

        let register = self.next_register();

        self.emit_instruction(instruction, operator_position);
        self.emit_instruction(Instruction::jump(1, true), operator_position);
        self.emit_instruction(
            Instruction::load_boolean(register, true, true),
            operator_position,
        );
        self.emit_instruction(
            Instruction::load_boolean(register, false, false),
            operator_position,
        );

        self.previous_expression_type = Type::Boolean;

        Ok(())
    }

    fn parse_logical_binary(&mut self) -> Result<(), CompileError> {
        let start_length = self.chunk.len();
        let (left_instruction, left_position) = self.pop_last_instruction()?;
        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(&operator);

        let test_instruction = match operator {
            Token::DoubleAmpersand => Instruction::test(left_instruction.a(), false),
            Token::DoublePipe => Instruction::test(left_instruction.a(), true),
            _ => {
                return Err(CompileError::ExpectedTokenMultiple {
                    expected: &[TokenKind::DoubleAmpersand, TokenKind::DoublePipe],
                    found: operator.to_owned(),
                    position: operator_position,
                })
            }
        };

        self.advance()?;
        self.emit_instruction(left_instruction, left_position);
        self.emit_instruction(test_instruction, operator_position);

        let jump_distance = (self.chunk.len() - start_length) as u8;

        self.emit_instruction(Instruction::jump(jump_distance, true), operator_position);
        self.parse_sub_expression(&rule.precedence)?;

        self.previous_expression_type = Type::Boolean;

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
            return self.parse_native_call(native_function);
        } else if Some(identifier) == self.chunk.name().map(|string| string.as_str()) {
            let register = self.next_register();

            self.emit_instruction(Instruction::load_self(register), start_position);
            self.declare_local(
                identifier,
                Type::SelfChunk,
                false,
                self.current_scope,
                register,
            );

            self.previous_expression_type = Type::SelfChunk;

            return Ok(());
        } else {
            return Err(CompileError::UndeclaredVariable {
                identifier: identifier.to_string(),
                position: start_position,
            });
        };

        let (is_mutable, local_scope) = {
            let local = self.get_local(local_index)?;

            (local.is_mutable, local.scope)
        };

        if !self.current_scope.contains(&local_scope) {
            return Err(CompileError::VariableOutOfScope {
                identifier: self.chunk.get_identifier(local_index).unwrap(),
                position: start_position,
                variable_scope: local_scope,
                access_scope: self.current_scope,
            });
        }

        if self.allow(Token::Equal)? {
            if !is_mutable {
                return Err(CompileError::CannotMutateImmutableVariable {
                    identifier: self.chunk.get_identifier(local_index).unwrap(),
                    position: start_position,
                });
            }

            self.parse_expression()?;

            let register = self.next_register() - 1;

            self.emit_instruction(
                Instruction::set_local(register, local_index),
                start_position,
            );

            self.previous_expression_type = Type::None;

            let mut optimizer = Optimizer::new(self.chunk.instructions_mut());
            let optimized = Optimizer::optimize_set_local(&mut optimizer);

            if optimized {
                self.optimization_count += 1;
            }

            return Ok(());
        }

        let register = self.next_register();

        self.emit_instruction(
            Instruction::get_local(register, local_index),
            self.previous_position,
        );

        let local = self.get_local(local_index)?;

        self.previous_expression_type = local.r#type.clone();

        Ok(())
    }

    fn parse_type_from(&mut self, token: Token, position: Span) -> Result<Type, CompileError> {
        match token {
            Token::Bool => Ok(Type::Boolean),
            Token::FloatKeyword => Ok(Type::Float),
            Token::Int => Ok(Type::Integer),
            Token::Str => Ok(Type::String { length: None }),
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

        self.block_index += 1;

        self.current_scope.begin(self.block_index);

        while !self.allow(Token::RightBrace)? && !self.is_eof() {
            self.parse(Precedence::None)?;
        }

        self.current_scope.end();

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

            if expected_register > start_register {
                if let Err(conflict) = item_type.check(&self.previous_expression_type) {
                    return Err(CompileError::ListItemTypeConflict {
                        conflict,
                        position: self.previous_position,
                    });
                }
            }

            item_type = self.previous_expression_type.clone();
            let actual_register = self.next_register() - 1;

            if expected_register < actual_register {
                self.emit_instruction(
                    Instruction::close(expected_register, actual_register),
                    self.current_position,
                );
            }

            self.allow(Token::Comma)?;
        }

        let to_register = self.next_register();
        let end = self.current_position.1;

        self.emit_instruction(
            Instruction::load_list(to_register, start_register),
            Span(start, end),
        );

        self.previous_expression_type = Type::List {
            item_type: Box::new(item_type),
            length: (to_register - start_register) as usize,
        };

        Ok(())
    }

    fn parse_if(&mut self) -> Result<(), CompileError> {
        self.advance()?;
        self.parse_expression()?;

        if matches!(
            self.get_last_operations(),
            Some([
                Operation::Equal | Operation::Less | Operation::LessEqual,
                Operation::Jump,
                Operation::LoadBoolean,
                Operation::LoadBoolean,
            ])
        ) {
            self.chunk.instructions_mut().pop();
            self.chunk.instructions_mut().pop();
            self.chunk.instructions_mut().pop();
        }

        let if_block_start = self.chunk.len();
        let if_block_start_position = self.current_position;

        if let Token::LeftBrace = self.current_token {
            self.parse_block()?;
        } else {
            return Err(CompileError::ExpectedToken {
                expected: TokenKind::LeftBrace,
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        }

        let if_block_end = self.chunk.len();
        let mut if_block_distance = (if_block_end - if_block_start) as u8;
        let if_block_type = self.previous_expression_type.clone();
        let if_last_register = self.next_register().saturating_sub(1);

        let has_else_statement = if let Token::Else = self.current_token {
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

            true
        } else if self.previous_expression_type != Type::None {
            return Err(CompileError::IfMissingElse {
                position: Span(if_block_start_position.0, self.current_position.1),
            });
        } else {
            false
        };

        let else_block_end = self.chunk.len();
        let else_block_distance = (else_block_end - if_block_end) as u8;

        if let Err(conflict) = if_block_type.check(&self.previous_expression_type) {
            return Err(CompileError::IfElseBranchMismatch {
                conflict,
                position: Span(if_block_start_position.0, self.current_position.1),
            });
        }

        match else_block_distance {
            0 => {}
            1 if !has_else_statement => {
                if let Some(skippable) = self.get_last_jumpable_mut() {
                    skippable.set_c_to_boolean(true);
                } else {
                    if_block_distance += 1;

                    self.chunk.instructions_mut().insert(
                        if_block_end,
                        (
                            Instruction::jump(else_block_distance, true),
                            self.current_position,
                        ),
                    );
                }
            }
            1 => {}
            2.. => {
                if_block_distance += 1;

                self.chunk.instructions_mut().insert(
                    if_block_end,
                    (
                        Instruction::jump(else_block_distance, true),
                        self.current_position,
                    ),
                );
            }
        }

        self.chunk.instructions_mut().insert(
            if_block_start,
            (
                Instruction::jump(if_block_distance, true),
                if_block_start_position,
            ),
        );

        if self.chunk.len() >= 4 {
            let mut optimizer = Optimizer::new(self.chunk.instructions_mut());
            let optimized = optimizer.optimize_comparison();

            if optimized {
                self.optimization_count += 1
            }
        }

        let else_last_register = self.next_register().saturating_sub(1);

        if if_last_register < else_last_register {
            self.emit_instruction(
                Instruction::r#move(else_last_register, if_last_register),
                self.current_position,
            );
        }

        Ok(())
    }

    fn parse_while(&mut self) -> Result<(), CompileError> {
        self.advance()?;

        let expression_start = self.chunk.len() as u8;

        self.parse_expression()?;

        if matches!(
            self.get_last_operations(),
            Some([
                Operation::Equal | Operation::Less | Operation::LessEqual,
                Operation::Jump,
                Operation::LoadBoolean,
                Operation::LoadBoolean,
            ],)
        ) {
            self.chunk.instructions_mut().pop();
            self.chunk.instructions_mut().pop();
            self.chunk.instructions_mut().pop();
        }

        let block_start = self.chunk.len();

        self.parse_block()?;

        let block_end = self.chunk.len() as u8;

        self.chunk.instructions_mut().insert(
            block_start,
            (
                Instruction::jump(block_end - block_start as u8 + 1, true),
                self.current_position,
            ),
        );

        let jump_back_distance = block_end - expression_start + 1;
        let jump_back = Instruction::jump(jump_back_distance, false);

        self.emit_instruction(jump_back, self.current_position);

        self.previous_expression_type = Type::None;

        Ok(())
    }

    fn parse_native_call(&mut self, function: NativeFunction) -> Result<(), CompileError> {
        let start = self.previous_position.0;
        let start_register = self.next_register();

        self.expect(Token::LeftParenthesis)?;

        while !self.allow(Token::RightParenthesis)? {
            let expected_register = self.next_register();

            self.parse_expression()?;

            let actual_register = self.next_register() - 1;

            if expected_register < actual_register {
                self.emit_instruction(
                    Instruction::close(expected_register, actual_register),
                    self.current_position,
                );
            }

            self.allow(Token::Comma)?;
        }

        let end = self.previous_position.1;
        let to_register = self.next_register();
        let argument_count = to_register - start_register;

        self.previous_expression_type = Type::Function(function.r#type());

        self.emit_instruction(
            Instruction::call_native(to_register, function, argument_count),
            Span(start, end),
        );
        Ok(())
    }

    fn parse_top_level(&mut self) -> Result<(), CompileError> {
        loop {
            self.parse(Precedence::None)?;

            if self.is_eof() || self.allow(Token::RightBrace)? {
                self.parse_implicit_return()?;

                break;
            }
        }

        Ok(())
    }

    fn parse_expression(&mut self) -> Result<(), CompileError> {
        self.parse(Precedence::None)?;

        if self.previous_expression_type == Type::None || self.chunk.is_empty() {
            return Err(CompileError::ExpectedExpression {
                found: self.previous_token.to_owned(),
                position: self.current_position,
            });
        }

        Ok(())
    }

    fn parse_sub_expression(&mut self, precedence: &Precedence) -> Result<(), CompileError> {
        self.parse(precedence.increment())
    }

    fn parse_return_statement(&mut self) -> Result<(), CompileError> {
        let start = self.current_position.0;

        self.advance()?;

        let has_return_value = if matches!(self.current_token, Token::Semicolon | Token::RightBrace)
        {
            false
        } else {
            self.parse_expression()?;

            true
        };
        let end = self.current_position.1;

        self.emit_instruction(Instruction::r#return(has_return_value), Span(start, end));

        self.previous_expression_type = Type::None;

        Ok(())
    }

    fn parse_implicit_return(&mut self) -> Result<(), CompileError> {
        if self.allow(Token::Semicolon)? {
            self.emit_instruction(Instruction::r#return(false), self.current_position);
        } else {
            self.emit_instruction(
                Instruction::r#return(self.previous_expression_type != Type::None),
                self.current_position,
            );
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

        let register = self.next_register() - 1;
        let r#type = if let Some(r#type) = explicit_type {
            r#type
        } else {
            self.get_register_type(register)?
        };
        let (local_index, _) =
            self.declare_local(identifier, r#type, is_mutable, self.current_scope, register);

        self.emit_instruction(
            Instruction::define_local(register, local_index, is_mutable),
            position,
        );

        self.previous_expression_type = Type::None;

        Ok(())
    }

    fn parse_function(&mut self) -> Result<(), CompileError> {
        let function_start = self.current_position.0;
        let mut function_compiler = Compiler::new(self.lexer)?;
        let identifier = if let Token::Identifier(text) = function_compiler.current_token {
            let position = function_compiler.current_position;

            function_compiler.advance()?;
            function_compiler.chunk.set_name(text.to_string());

            Some((text, position))
        } else {
            None
        };

        function_compiler.expect(Token::LeftParenthesis)?;

        let mut value_parameters: Option<Vec<(u8, Type)>> = None;

        while function_compiler.current_token != Token::RightParenthesis {
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

            let register = function_compiler.next_register();

            let (_, identifier_index) = function_compiler.declare_local(
                parameter,
                r#type.clone(),
                is_mutable,
                function_compiler.current_scope,
                register,
            );

            if let Some(value_parameters) = value_parameters.as_mut() {
                value_parameters.push((identifier_index, r#type));
            } else {
                value_parameters = Some(vec![(identifier_index, r#type)]);
            };

            function_compiler.minimum_register += 1;

            function_compiler.allow(Token::Comma)?;
        }

        function_compiler.advance()?;

        let return_type = if function_compiler.allow(Token::ArrowThin)? {
            let r#type = function_compiler.parse_type_from(
                function_compiler.current_token,
                function_compiler.current_position,
            )?;

            function_compiler.advance()?;

            Box::new(r#type)
        } else {
            Box::new(Type::None)
        };

        function_compiler.expect(Token::LeftBrace)?;
        function_compiler.parse_top_level()?;

        self.previous_token = function_compiler.previous_token;
        self.previous_position = function_compiler.previous_position;
        self.current_token = function_compiler.current_token;
        self.current_position = function_compiler.current_position;

        let function_type = FunctionType {
            type_parameters: None,
            value_parameters,
            return_type,
        };
        let function = ConcreteValue::Function(function_compiler.finish());
        let constant_index = self.chunk.push_or_get_constant(function);
        let function_end = self.current_position.1;
        let register = self.next_register();

        self.lexer.skip_to(function_end);

        if let Some((identifier, identifier_position)) = identifier {
            let (local_index, _) = self.declare_local(
                identifier,
                Type::Function(function_type),
                false,
                self.current_scope,
                register,
            );

            self.emit_instruction(
                Instruction::load_constant(register, constant_index, false),
                Span(function_start, function_end),
            );
            self.emit_instruction(
                Instruction::define_local(register, local_index, false),
                identifier_position,
            );

            self.previous_expression_type = Type::None;
        } else {
            self.emit_instruction(
                Instruction::load_constant(register, constant_index, false),
                Span(function_start, function_end),
            );

            self.previous_expression_type = Type::Function(function_type);
        }

        Ok(())
    }

    fn parse_call(&mut self) -> Result<(), CompileError> {
        let (last_instruction, _) =
            self.chunk
                .instructions()
                .last()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;

        if !last_instruction.yields_value() {
            return Err(CompileError::ExpectedExpression {
                found: self.previous_token.to_owned(),
                position: self.previous_position,
            });
        }

        let function_register = last_instruction.a();
        let register_type = self.get_register_type(function_register)?;
        let function_return_type = match register_type {
            Type::Function(function_type) => *function_type.return_type,
            Type::SelfChunk => (*self.chunk.r#type().return_type).clone(),
            _ => {
                return Err(CompileError::ExpectedFunction {
                    found: self.previous_token.to_owned(),
                    actual_type: register_type,
                    position: self.previous_position,
                });
            }
        };
        let start = self.current_position.0;

        self.advance()?;

        while !self.allow(Token::RightParenthesis)? {
            let expected_register = self.next_register();

            self.parse_expression()?;

            let actual_register = self.next_register() - 1;

            if expected_register < actual_register {
                self.emit_instruction(
                    Instruction::close(expected_register, actual_register),
                    self.current_position,
                );
            }

            self.allow(Token::Comma)?;
        }

        let end = self.current_position.1;
        let to_register = self.next_register();
        let argument_count = to_register - function_register - 1;

        self.emit_instruction(
            Instruction::call(to_register, function_register, argument_count),
            Span(start, end),
        );

        self.previous_expression_type = function_return_type;

        Ok(())
    }

    fn parse_semicolon(&mut self) -> Result<(), CompileError> {
        self.advance()?;

        self.previous_expression_type = Type::None;

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
            log::debug!(
                "{} is prefix with precedence {precedence}",
                self.current_token.to_string().bold(),
            );

            prefix_parser(self)?;
        }

        let mut infix_rule = ParseRule::from(&self.current_token);

        while precedence <= infix_rule.precedence {
            if let Some(infix_parser) = infix_rule.infix {
                log::debug!(
                    "{} is infix with precedence {precedence}",
                    self.current_token.to_string().bold(),
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

/// Operator precedence levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None,
    Assignment,
    Conditional,
    LogicalOr,
    LogicalAnd,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    fn increment(&self) -> Self {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Conditional,
            Precedence::Conditional => Precedence::LogicalOr,
            Precedence::LogicalOr => Precedence::LogicalAnd,
            Precedence::LogicalAnd => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
        }
    }
}

impl Display for Precedence {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

type Parser<'a> = fn(&mut Compiler<'a>) -> Result<(), CompileError>;

/// Rule that defines how to parse a token.
#[derive(Debug, Clone, Copy)]
struct ParseRule<'a> {
    pub prefix: Option<Parser<'a>>,
    pub infix: Option<Parser<'a>>,
    pub precedence: Precedence,
}

impl From<&Token<'_>> for ParseRule<'_> {
    fn from(token: &Token) -> Self {
        match token {
            Token::ArrowThin => ParseRule {
                prefix: Some(Compiler::expect_expression),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Async => todo!(),
            Token::Bang => ParseRule {
                prefix: Some(Compiler::parse_unary),
                infix: None,
                precedence: Precedence::Unary,
            },
            Token::BangEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::parse_comparison_binary),
                precedence: Precedence::Equality,
            },
            Token::Bool => ParseRule {
                prefix: Some(Compiler::expect_expression),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Boolean(_) => ParseRule {
                prefix: Some(Compiler::parse_boolean),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Break => todo!(),
            Token::Byte(_) => ParseRule {
                prefix: Some(Compiler::parse_byte),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Character(_) => ParseRule {
                prefix: Some(Compiler::parse_character),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Colon => ParseRule {
                prefix: Some(Compiler::expect_expression),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Comma => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::Dot => ParseRule {
                prefix: Some(Compiler::expect_expression),
                infix: None,
                precedence: Precedence::None,
            },
            Token::DoubleAmpersand => ParseRule {
                prefix: None,
                infix: Some(Compiler::parse_logical_binary),
                precedence: Precedence::LogicalAnd,
            },
            Token::DoubleEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::parse_comparison_binary),
                precedence: Precedence::Equality,
            },
            Token::DoublePipe => ParseRule {
                prefix: None,
                infix: Some(Compiler::parse_logical_binary),
                precedence: Precedence::LogicalOr,
            },
            Token::DoubleDot => ParseRule {
                prefix: Some(Compiler::expect_expression),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Eof => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::Equal => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::Assignment,
            },
            Token::Else => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::Float(_) => ParseRule {
                prefix: Some(Compiler::parse_float),
                infix: None,
                precedence: Precedence::None,
            },
            Token::FloatKeyword => ParseRule {
                prefix: Some(Compiler::expect_expression),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Fn => ParseRule {
                prefix: Some(Compiler::parse_function),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Greater => ParseRule {
                prefix: None,
                infix: Some(Compiler::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            Token::GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            Token::Identifier(_) => ParseRule {
                prefix: Some(Compiler::parse_variable),
                infix: None,
                precedence: Precedence::None,
            },
            Token::If => ParseRule {
                prefix: Some(Compiler::parse_if),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Int => ParseRule {
                prefix: Some(Compiler::expect_expression),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Integer(_) => ParseRule {
                prefix: Some(Compiler::parse_integer),
                infix: None,
                precedence: Precedence::None,
            },
            Token::LeftBrace => ParseRule {
                prefix: Some(Compiler::parse_block),
                infix: None,
                precedence: Precedence::None,
            },
            Token::LeftParenthesis => ParseRule {
                prefix: Some(Compiler::parse_grouped),
                infix: Some(Compiler::parse_call),
                precedence: Precedence::Call,
            },
            Token::LeftBracket => ParseRule {
                prefix: Some(Compiler::parse_list),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Less => ParseRule {
                prefix: None,
                infix: Some(Compiler::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            Token::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            Token::Let => ParseRule {
                prefix: Some(Compiler::parse_let_statement),
                infix: None,
                precedence: Precedence::Assignment,
            },
            Token::Loop => todo!(),
            Token::Map => todo!(),
            Token::Minus => ParseRule {
                prefix: Some(Compiler::parse_unary),
                infix: Some(Compiler::parse_math_binary),
                precedence: Precedence::Term,
            },
            Token::MinusEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::parse_math_binary),
                precedence: Precedence::Assignment,
            },
            Token::Mut => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::Percent => ParseRule {
                prefix: None,
                infix: Some(Compiler::parse_math_binary),
                precedence: Precedence::Factor,
            },
            Token::PercentEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::parse_math_binary),
                precedence: Precedence::Assignment,
            },
            Token::Plus => ParseRule {
                prefix: None,
                infix: Some(Compiler::parse_math_binary),
                precedence: Precedence::Term,
            },
            Token::PlusEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::parse_math_binary),
                precedence: Precedence::Assignment,
            },
            Token::Return => ParseRule {
                prefix: Some(Compiler::parse_return_statement),
                infix: None,
                precedence: Precedence::None,
            },
            Token::RightBrace => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::RightParenthesis => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::RightBracket => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::Semicolon => ParseRule {
                prefix: Some(Compiler::parse_semicolon),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Slash => ParseRule {
                prefix: None,
                infix: Some(Compiler::parse_math_binary),
                precedence: Precedence::Factor,
            },
            Token::SlashEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::parse_math_binary),
                precedence: Precedence::Assignment,
            },
            Token::Star => ParseRule {
                prefix: None,
                infix: Some(Compiler::parse_math_binary),
                precedence: Precedence::Factor,
            },
            Token::StarEqual => ParseRule {
                prefix: None,
                infix: Some(Compiler::parse_math_binary),
                precedence: Precedence::Assignment,
            },
            Token::Str => ParseRule {
                prefix: Some(Compiler::expect_expression),
                infix: None,
                precedence: Precedence::None,
            },
            Token::String(_) => ParseRule {
                prefix: Some(Compiler::parse_string),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Struct => todo!(),
            Token::While => ParseRule {
                prefix: Some(Compiler::parse_while),
                infix: None,
                precedence: Precedence::None,
            },
        }
    }
}

/// Compilation errors
#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    // Token errors
    ExpectedToken {
        expected: TokenKind,
        found: TokenOwned,
        position: Span,
    },
    ExpectedTokenMultiple {
        expected: &'static [TokenKind],
        found: TokenOwned,
        position: Span,
    },

    // Parsing errors
    CannotChainComparison {
        position: Span,
    },
    ExpectedExpression {
        found: TokenOwned,
        position: Span,
    },
    ExpectedFunction {
        found: TokenOwned,
        actual_type: Type,
        position: Span,
    },
    InvalidAssignmentTarget {
        found: TokenOwned,
        position: Span,
    },
    UnexpectedReturn {
        position: Span,
    },

    // Variable errors
    CannotMutateImmutableVariable {
        identifier: String,
        position: Span,
    },
    ExpectedMutableVariable {
        found: TokenOwned,
        position: Span,
    },
    UndeclaredVariable {
        identifier: String,
        position: Span,
    },
    VariableOutOfScope {
        identifier: String,
        variable_scope: Scope,
        access_scope: Scope,
        position: Span,
    },

    // Type errors
    CannotResolveRegisterType {
        register_index: usize,
        position: Span,
    },
    CannotResolveVariableType {
        identifier: String,
        position: Span,
    },
    IfElseBranchMismatch {
        conflict: TypeConflict,
        position: Span,
    },
    IfMissingElse {
        position: Span,
    },
    ListItemTypeConflict {
        conflict: TypeConflict,
        position: Span,
    },

    // Wrappers around foreign errors
    Chunk {
        error: ChunkError,
        position: Span,
    },
    Lex(LexError),
    ParseFloatError {
        error: ParseFloatError,
        position: Span,
    },
    ParseIntError {
        error: ParseIntError,
        position: Span,
    },
}

impl AnnotatedError for CompileError {
    fn title() -> &'static str {
        "Compilation Error"
    }

    fn description(&self) -> &'static str {
        match self {
            Self::CannotChainComparison { .. } => "Cannot chain comparison operations",
            Self::CannotMutateImmutableVariable { .. } => "Cannot mutate immutable variable",
            Self::CannotResolveRegisterType { .. } => "Cannot resolve register type",
            Self::CannotResolveVariableType { .. } => "Cannot resolve type",
            Self::Chunk { .. } => "Chunk error",
            Self::ExpectedExpression { .. } => "Expected an expression",
            Self::ExpectedFunction { .. } => "Expected a function",
            Self::ExpectedMutableVariable { .. } => "Expected a mutable variable",
            Self::ExpectedToken { .. } => "Expected a specific token",
            Self::ExpectedTokenMultiple { .. } => "Expected one of multiple tokens",
            Self::IfElseBranchMismatch { .. } => "Type mismatch in if/else branches",
            Self::IfMissingElse { .. } => "If statement missing else branch",
            Self::InvalidAssignmentTarget { .. } => "Invalid assignment target",
            Self::Lex(error) => error.description(),
            Self::ListItemTypeConflict { .. } => "List item type conflict",
            Self::ParseFloatError { .. } => "Failed to parse float",
            Self::ParseIntError { .. } => "Failed to parse integer",
            Self::UndeclaredVariable { .. } => "Undeclared variable",
            Self::UnexpectedReturn { .. } => "Unexpected return",
            Self::VariableOutOfScope { .. } => "Variable out of scope",
        }
    }

    fn details(&self) -> Option<String> {
        match self {
            Self::CannotMutateImmutableVariable { identifier, .. } => {
                Some(format!("{identifier} is immutable"))
            }
            Self::Chunk { error, .. } => Some(error.to_string()),
            Self::ExpectedExpression { found, .. } => Some(format!("Found {found}")),
            Self::ExpectedFunction { found, actual_type, .. } => {
                Some(format!("Expected \"{found}\" to be a function but it has type {actual_type}"))
            }
            Self::ExpectedToken {
                expected, found, ..
            } => Some(format!("Expected {expected} but found {found}")),
            Self::ExpectedTokenMultiple {
                expected, found, ..
            } => {
                let mut details = String::from("Expected");

                for (index, token) in expected.iter().enumerate() {
                    details.push_str(&format!(" {token}"));

                    if index < expected.len() - 2 {
                        details.push_str(", ");
                    }

                    if index == expected.len() - 2 {
                        details.push_str(" or");
                    }
                }

                details.push_str(&format!(" but found {found}"));

                Some(details)
            }
            Self::ExpectedMutableVariable { found, .. } => Some(format!("Found {found}")),
            Self::IfElseBranchMismatch {
                conflict: TypeConflict { expected, actual },
                ..
            } => Some(
                format!("This if block evaluates to type \"{expected}\" but the else block evaluates to \"{actual}\"")
            ),
            Self::IfMissingElse { .. } => Some(
                "This \"if\" expression evaluates to a value but is missing an else block"
                    .to_string(),
            ),
            Self::InvalidAssignmentTarget { found, .. } => {
                Some(format!("Cannot assign to {found}"))
            }
            Self::Lex(error) => error.details(),
            Self::ParseFloatError { error, .. } => Some(error.to_string()),
            Self::ParseIntError { error, .. } => Some(error.to_string()),
            Self::UndeclaredVariable { identifier, .. } => {
                Some(format!("{identifier} has not been declared"))
            }
            Self::UnexpectedReturn { .. } => None,
            Self::VariableOutOfScope { identifier, .. } => {
                Some(format!("{identifier} is out of scope"))
            }
            _ => None,
        }
    }

    fn position(&self) -> Span {
        match self {
            Self::CannotChainComparison { position } => *position,
            Self::CannotMutateImmutableVariable { position, .. } => *position,
            Self::CannotResolveRegisterType { position, .. } => *position,
            Self::CannotResolveVariableType { position, .. } => *position,
            Self::Chunk { position, .. } => *position,
            Self::ExpectedExpression { position, .. } => *position,
            Self::ExpectedFunction { position, .. } => *position,
            Self::ExpectedMutableVariable { position, .. } => *position,
            Self::ExpectedToken { position, .. } => *position,
            Self::ExpectedTokenMultiple { position, .. } => *position,
            Self::IfElseBranchMismatch { position, .. } => *position,
            Self::IfMissingElse { position } => *position,
            Self::InvalidAssignmentTarget { position, .. } => *position,
            Self::Lex(error) => error.position(),
            Self::ListItemTypeConflict { position, .. } => *position,
            Self::ParseFloatError { position, .. } => *position,
            Self::ParseIntError { position, .. } => *position,
            Self::UndeclaredVariable { position, .. } => *position,
            Self::UnexpectedReturn { position } => *position,
            Self::VariableOutOfScope { position, .. } => *position,
        }
    }
}

impl From<LexError> for CompileError {
    fn from(error: LexError) -> Self {
        Self::Lex(error)
    }
}
