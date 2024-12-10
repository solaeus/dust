//! Compilation tools and errors
//!
//! This module provides two compilation options:
//! - [`compile`] borrows a string and returns a chunk, handling the entire compilation process and
//!   turning any resulting [`ComplileError`] into a [`DustError`].
//! - [`Compiler`] uses a lexer to get tokens and assembles a chunk.
mod optimize;

use std::{
    fmt::{self, Display, Formatter},
    mem::replace,
    num::{ParseFloatError, ParseIntError},
};

use colored::Colorize;
use optimize::{
    condense_set_local_to_math, optimize_test_with_explicit_booleans,
    optimize_test_with_loader_arguments,
};
use smallvec::{smallvec, SmallVec};

use crate::{
    instruction::{
        Call, CallNative, Close, GetLocal, Jump, LoadConstant, LoadList, LoadSelf, Move, Negate,
        Not, Return, SetLocal, Test,
    },
    AnnotatedError, Argument, Chunk, ConcreteValue, DustError, DustString, FunctionType,
    Instruction, LexError, Lexer, Local, NativeFunction, Operation, Scope, Span, Token, TokenKind,
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
/// assert_eq!(chunk.len(), 3);
/// ```
pub fn compile(source: &str) -> Result<Chunk, DustError> {
    let lexer = Lexer::new(source);
    let mut compiler = Compiler::new(lexer).map_err(|error| DustError::compile(error, source))?;

    compiler
        .compile()
        .map_err(|error| DustError::compile(error, source))?;

    let chunk = compiler.finish(None, None);

    Ok(chunk)
}

/// Tool for compiling the input a token at a time while assembling a chunk.
///
/// See the [`compile`] function an example of how to create and use a Compiler.
#[derive(Debug)]
pub struct Compiler<'src> {
    self_name: Option<DustString>,
    instructions: SmallVec<[(Instruction, Type, Span); 32]>,
    constants: SmallVec<[ConcreteValue; 16]>,
    locals: SmallVec<[(Local, Type); 8]>,

    lexer: Lexer<'src>,

    current_token: Token<'src>,
    current_position: Span,
    previous_token: Token<'src>,
    previous_position: Span,

    return_type: Option<Type>,
    minimum_register: u8,
    block_index: u8,
    current_scope: Scope,
}

impl<'src> Compiler<'src> {
    pub fn new(mut lexer: Lexer<'src>) -> Result<Self, CompileError> {
        let (current_token, current_position) = lexer.next_token()?;

        log::info!(
            "Begin chunk with {} at {}",
            current_token.to_string().bold(),
            current_position.to_string()
        );

        Ok(Compiler {
            self_name: None,
            instructions: SmallVec::new(),
            constants: SmallVec::new(),
            locals: SmallVec::new(),
            lexer,
            current_token,
            current_position,
            previous_token: Token::Eof,
            previous_position: Span(0, 0),
            return_type: None,
            minimum_register: 0,
            block_index: 0,
            current_scope: Scope::default(),
        })
    }

    pub fn finish(
        self,
        type_parameters: Option<SmallVec<[u8; 4]>>,
        value_parameters: Option<SmallVec<[(u8, Type); 4]>>,
    ) -> Chunk {
        log::info!("End chunk");

        let r#type = FunctionType {
            type_parameters,
            value_parameters,
            return_type: self.return_type.unwrap_or(Type::None),
        };
        let instructions = self
            .instructions
            .into_iter()
            .map(|(instruction, _, position)| (instruction, position))
            .collect::<SmallVec<[(Instruction, Span); 32]>>();
        let locals = self
            .locals
            .into_iter()
            .map(|(local, _)| local)
            .collect::<SmallVec<[Local; 8]>>();

        Chunk::with_data(self.self_name, r#type, instructions, self.constants, locals)
    }

    pub fn compile(&mut self) -> Result<(), CompileError> {
        loop {
            self.parse(Precedence::None)?;

            if matches!(self.current_token, Token::Eof | Token::RightBrace) {
                self.parse_implicit_return()?;

                break;
            }
        }

        Ok(())
    }

    fn is_eof(&self) -> bool {
        matches!(self.current_token, Token::Eof)
    }

    fn next_register(&self) -> u8 {
        self.instructions
            .iter()
            .rev()
            .find_map(|(instruction, _, _)| {
                if instruction.yields_value() {
                    Some(instruction.a + 1)
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

    fn get_local(&self, index: u8) -> Result<&(Local, Type), CompileError> {
        self.locals
            .get(index as usize)
            .ok_or(CompileError::UndeclaredVariable {
                identifier: format!("#{}", index),
                position: self.current_position,
            })
    }

    fn get_local_index(&self, identifier_text: &str) -> Result<u8, CompileError> {
        self.locals
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, (local, _))| {
                let constant = self.constants.get(local.identifier_index as usize)?;
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
        register_index: u8,
        r#type: Type,
        is_mutable: bool,
        scope: Scope,
    ) -> (u8, u8) {
        log::info!("Declaring local {identifier}");

        let identifier = ConcreteValue::string(identifier);
        let identifier_index = self.push_or_get_constant(identifier);
        let local_index = self.locals.len() as u8;

        self.locals.push((
            Local::new(identifier_index, register_index, is_mutable, scope),
            r#type,
        ));

        (local_index, identifier_index)
    }

    fn get_identifier(&self, local_index: u8) -> Option<String> {
        self.locals
            .get(local_index as usize)
            .and_then(|(local, _)| {
                self.constants
                    .get(local.identifier_index as usize)
                    .map(|value| value.to_string())
            })
    }

    fn push_or_get_constant(&mut self, value: ConcreteValue) -> u8 {
        if let Some(index) = self
            .constants
            .iter()
            .position(|constant| constant == &value)
        {
            index as u8
        } else {
            let index = self.constants.len() as u8;

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

    fn pop_last_instruction(&mut self) -> Result<(Instruction, Type, Span), CompileError> {
        self.instructions
            .pop()
            .ok_or_else(|| CompileError::ExpectedExpression {
                found: self.previous_token.to_owned(),
                position: self.previous_position,
            })
    }

    fn get_last_operations<const COUNT: usize>(&self) -> Option<[Operation; COUNT]> {
        let mut n_operations = [Operation::Return; COUNT];

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

    fn get_last_jumpable_mut_between(
        &mut self,
        minimum: usize,
        maximum: usize,
    ) -> Option<&mut Instruction> {
        self.instructions
            .iter_mut()
            .rev()
            .skip(minimum)
            .take(maximum)
            .find_map(|(instruction, _, _)| {
                if let Operation::LoadBoolean | Operation::LoadConstant = instruction.operation() {
                    Some(instruction)
                } else {
                    None
                }
            })
    }

    fn get_last_instruction_type(&self) -> Type {
        self.instructions
            .last()
            .map(|(_, r#type, _)| r#type.clone())
            .unwrap_or(Type::None)
    }

    fn get_register_type(&self, register_index: u8) -> Result<Type, CompileError> {
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

            if let Operation::LoadList = operation {
                let LoadList { start_register, .. } = LoadList::from(instruction);
                let item_type = self.get_register_type(start_register)?;

                return Ok(Type::List(Box::new(item_type)));
            }

            if let Operation::LoadSelf = operation {
                return Ok(Type::SelfChunk);
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

    /// Updates [Self::return_type] with the given [Type].
    ///
    /// If [Self::return_type] is already set, it will check if the given [Type] is compatible with
    /// it and set it to the least restrictive of the two.
    fn update_return_type(&mut self, new_return_type: Type) -> Result<(), CompileError> {
        if let Some(return_type) = &self.return_type {
            return_type.check(&new_return_type).map_err(|conflict| {
                CompileError::ReturnTypeConflict {
                    conflict,
                    position: self.current_position,
                }
            })?;

            if *return_type != Type::Any {
                self.return_type = Some(new_return_type);
            };
        } else {
            self.return_type = Some(new_return_type);
        }

        Ok(())
    }

    fn emit_instruction(&mut self, instruction: Instruction, r#type: Type, position: Span) {
        log::debug!(
            "Emitting {} at {}",
            instruction.operation().to_string().bold(),
            position.to_string()
        );

        self.instructions.push((instruction, r#type, position));
    }

    fn emit_constant(
        &mut self,
        constant: ConcreteValue,
        position: Span,
    ) -> Result<(), CompileError> {
        let r#type = constant.r#type();
        let constant_index = self.push_or_get_constant(constant);
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
            self.pop_last_instruction()?;
        let (argument, push_back) = self.handle_binary_argument(&previous_instruction)?;

        if push_back {
            self.instructions.push((
                previous_instruction,
                previous_type.clone(),
                previous_position,
            ))
        }

        let destination = self.next_register();
        let instruction = match operator.kind() {
            TokenKind::Bang => Instruction::from(Not {
                destination,
                argument,
            }),
            TokenKind::Minus => Instruction::from(Negate {
                destination,
                argument,
            }),
            _ => {
                return Err(CompileError::ExpectedTokenMultiple {
                    expected: &[TokenKind::Bang, TokenKind::Minus],
                    found: operator.to_owned(),
                    position: operator_position,
                })
            }
        };

        self.emit_instruction(instruction, previous_type, operator_position);

        Ok(())
    }

    fn handle_binary_argument(
        &mut self,
        instruction: &Instruction,
    ) -> Result<(Argument, bool), CompileError> {
        let argument =
            instruction
                .as_argument()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;
        let push_back = matches!(argument, Argument::Register(_));

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
        let left_is_mutable_local = if let Operation::GetLocal = left_instruction.operation() {
            let GetLocal { local_index, .. } = GetLocal::from(&left_instruction);

            self.locals
                .get(local_index as usize)
                .map(|(local, _)| local.is_mutable)
                .unwrap_or(false)
        } else {
            false
        };
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

        if push_back_left {
            self.instructions
                .push((left_instruction, left_type.clone(), left_position));
        }

        if is_assignment && !left_is_mutable_local {
            return Err(CompileError::ExpectedMutableVariable {
                found: self.previous_token.to_owned(),
                position: left_position,
            });
        }

        match operator {
            Token::Plus | Token::PlusEqual => {
                Compiler::expect_addable_type(&left_type, &left_position)?
            }
            Token::Minus | Token::MinusEqual => {
                Compiler::expect_subtractable_type(&left_type, &left_position)?
            }
            Token::Slash | Token::SlashEqual => {
                Compiler::expect_dividable_type(&left_type, &left_position)?
            }
            Token::Star | Token::StarEqual => {
                Compiler::expect_multipliable_type(&left_type, &left_position)?
            }
            Token::Percent | Token::PercentEqual => {
                Compiler::expect_modulable_type(&left_type, &left_position)?
            }
            _ => {}
        }

        let r#type = if is_assignment {
            Type::None
        } else if left_type == Type::Character {
            Type::String
        } else {
            left_type.clone()
        };

        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        let (right_instruction, right_type, right_position) = self.pop_last_instruction()?;
        let (right, push_back_right) = self.handle_binary_argument(&right_instruction)?;

        match operator {
            Token::Plus | Token::PlusEqual => {
                Compiler::expect_addable_type(&right_type, &right_position)?;
                Compiler::expect_addable_types(
                    &left_type,
                    &left_position,
                    &right_type,
                    &right_position,
                )?;
            }
            Token::Minus | Token::MinusEqual => {
                Compiler::expect_subtractable_type(&right_type, &right_position)?;
                Compiler::expect_subtractable_types(
                    &left_type,
                    &left_position,
                    &right_type,
                    &right_position,
                )?;
            }
            Token::Slash | Token::SlashEqual => {
                Compiler::expect_dividable_type(&right_type, &right_position)?;
                Compiler::expect_dividable_types(
                    &left_type,
                    &left_position,
                    &right_type,
                    &right_position,
                )?;
            }
            Token::Star | Token::StarEqual => {
                Compiler::expect_multipliable_type(&right_type, &right_position)?;
                Compiler::expect_multipliable_types(
                    &left_type,
                    &left_position,
                    &right_type,
                    &right_position,
                )?;
            }
            Token::Percent | Token::PercentEqual => {
                Compiler::expect_modulable_type(&right_type, &right_position)?;
                Compiler::expect_modulable_types(
                    &left_type,
                    &left_position,
                    &right_type,
                    &right_position,
                )?;
            }
            _ => {}
        }

        if push_back_right {
            self.instructions
                .push((right_instruction, right_type, right_position));
        }

        let destination = if is_assignment {
            match left {
                Argument::Register(register) => register,
                Argument::Constant(_) => self.next_register(),
            }
        } else {
            self.next_register()
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
                })
            }
        };

        self.emit_instruction(instruction, r#type, operator_position);

        Ok(())
    }

    fn parse_comparison_binary(&mut self) -> Result<(), CompileError> {
        if let Some([Operation::Equal | Operation::Less | Operation::LessEqual, _, _]) =
            self.get_last_operations()
        {
            return Err(CompileError::CannotChainComparison {
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

        if push_back_right {
            self.instructions
                .push((right_instruction, right_type, right_position));
        }

        let destination = self.next_register();
        let comparison = match operator {
            Token::DoubleEqual => Instruction::equal(destination, true, left, right),
            Token::BangEqual => Instruction::equal(destination, false, left, right),
            Token::Less => Instruction::less(destination, true, left, right),
            Token::LessEqual => Instruction::less_equal(destination, true, left, right),
            Token::Greater => Instruction::less_equal(destination, false, left, right),
            Token::GreaterEqual => Instruction::less(destination, false, left, right),
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

        self.emit_instruction(comparison, Type::Boolean, operator_position);

        Ok(())
    }

    fn parse_logical_binary(&mut self) -> Result<(), CompileError> {
        let is_logic_chain = matches!(
            self.get_last_operations(),
            Some([Operation::Test, Operation::Jump, _])
        );

        let (mut left_instruction, left_type, left_position) = self.pop_last_instruction()?;

        if is_logic_chain {
            let destination = self
                .instructions
                .iter()
                .rev()
                .nth(2)
                .map_or(0, |(instruction, _, _)| instruction.a);

            left_instruction.a = destination;
        }

        let jump_index = self.instructions.len().saturating_sub(1);
        let mut jump_distance = if is_logic_chain {
            self.instructions.pop().map_or(0, |(jump, _, _)| {
                let Jump { offset, .. } = Jump::from(&jump);

                offset
            })
        } else {
            0
        };

        if !left_instruction.yields_value() {
            return Err(CompileError::ExpectedExpression {
                found: self.previous_token.to_owned(),
                position: self.previous_position,
            });
        }

        let (left, _) = self.handle_binary_argument(&left_instruction)?;

        self.instructions
            .push((left_instruction, left_type.clone(), left_position));

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
                })
            }
        };
        let test = Instruction::test(left, test_boolean);

        self.advance()?;
        self.emit_instruction(test, Type::None, operator_position);
        self.emit_instruction(Instruction::jump(1, true), Type::None, operator_position);
        self.parse_sub_expression(&rule.precedence)?;

        let (mut right_instruction, _, _) = self.instructions.last_mut().unwrap();
        right_instruction.a = left_instruction.a;

        if is_logic_chain {
            let expression_length = self.instructions.len() - jump_index - 1;
            jump_distance += expression_length as u8;
            let jump = Instruction::jump(jump_distance, true);

            self.instructions
                .insert(jump_index, (jump, Type::None, operator_position));
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
        } else if self.self_name.as_deref() == Some(identifier) {
            let destination = self.next_register();
            let load_self = Instruction::from(LoadSelf { destination });

            self.emit_instruction(load_self, Type::SelfChunk, start_position);

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

            let register = self.next_register() - 1;
            let set_local = Instruction::from(SetLocal {
                register_index: register,
                local_index,
            });

            self.emit_instruction(set_local, Type::None, start_position);
            condense_set_local_to_math(self)?;

            return Ok(());
        }

        let destination = self.next_register();
        let get_local = Instruction::from(GetLocal {
            destination,
            local_index,
        });

        self.emit_instruction(get_local, r#type, self.previous_position);

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
        let load_list = Instruction::from(LoadList {
            destination,
            start_register,
        });

        self.emit_instruction(load_list, Type::List(Box::new(item_type)), Span(start, end));

        Ok(())
    }

    fn parse_if(&mut self) -> Result<(), CompileError> {
        self.advance()?;
        self.parse_expression()?;

        if let Some((instruction, _, _)) = self.instructions.last() {
            let argument = match instruction.as_argument() {
                Some(argument) => argument,
                None => {
                    return Err(CompileError::ExpectedExpression {
                        found: self.previous_token.to_owned(),
                        position: self.previous_position,
                    });
                }
            };
            let test = Instruction::from(Test {
                argument,
                test_value: true,
            });

            self.emit_instruction(test, Type::None, self.current_position)
        }

        let if_block_start = self.instructions.len();
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

        let if_block_end = self.instructions.len();
        let mut if_block_distance = (if_block_end - if_block_start) as u8;
        let if_block_type = self.get_last_instruction_type();
        let if_last_register = self.next_register().saturating_sub(1);

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

            true
        } else if if_block_type != Type::None {
            return Err(CompileError::IfMissingElse {
                position: Span(if_block_start_position.0, self.current_position.1),
            });
        } else {
            false
        };

        let else_block_end = self.instructions.len();
        let else_block_distance = (else_block_end - if_block_end) as u8;
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
                if let Some(skippable) =
                    self.get_last_jumpable_mut_between(1, if_block_distance as usize)
                {
                    skippable.c = true as u8;
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

        let jump = Instruction::from(Jump {
            offset: if_block_distance,
            is_positive: true,
        });

        self.instructions
            .insert(if_block_start, (jump, Type::None, if_block_start_position));

        optimize_test_with_explicit_booleans(self);
        optimize_test_with_loader_arguments(self);

        let else_last_register = self.next_register().saturating_sub(1);
        let r#move = Instruction::from(Move {
            from: else_last_register,
            to: if_last_register,
        });

        if if_last_register < else_last_register {
            self.emit_instruction(r#move, Type::None, self.current_position);
        }

        Ok(())
    }

    fn parse_while(&mut self) -> Result<(), CompileError> {
        self.advance()?;

        let expression_start = self.instructions.len() as u8;

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
            self.instructions.pop();
            self.instructions.pop();
            self.instructions.pop();
        }

        let block_start = self.instructions.len();

        self.parse_block()?;

        let block_end = self.instructions.len() as u8;
        let jump_distance = block_end - block_start as u8 + 1;
        let jump = Instruction::from(Jump {
            offset: jump_distance,
            is_positive: true,
        });

        self.instructions
            .insert(block_start, (jump, Type::None, self.current_position));

        let jump_back_distance = block_end - expression_start + 1;
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
        let r#return = Instruction::from(Return {
            should_return_value,
        });

        self.emit_instruction(r#return, Type::None, Span(start, end));

        Ok(())
    }

    fn parse_implicit_return(&mut self) -> Result<(), CompileError> {
        if self.allow(Token::Semicolon)? {
            let r#return = Instruction::from(Return {
                should_return_value: false,
            });

            self.emit_instruction(r#return, Type::None, self.current_position);
        } else {
            let previous_expression_type =
                self.instructions
                    .last()
                    .map_or(Type::None, |(instruction, r#type, _)| {
                        if instruction.yields_value() {
                            r#type.clone()
                        } else {
                            Type::None
                        }
                    });
            let should_return_value = previous_expression_type != Type::None;
            let r#return = Instruction::from(Return {
                should_return_value,
            });

            self.update_return_type(previous_expression_type.clone())?;
            self.emit_instruction(r#return, Type::None, self.current_position);
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

        Ok(())
    }

    fn parse_function(&mut self) -> Result<(), CompileError> {
        let function_start = self.current_position.0;
        let mut function_compiler = Compiler::new(self.lexer)?;
        let identifier_info = if let Token::Identifier(text) = function_compiler.current_token {
            let position = function_compiler.current_position;

            function_compiler.advance()?;

            function_compiler.self_name = Some(text.into());

            Some((text, position))
        } else {
            None
        };

        function_compiler.expect(Token::LeftParenthesis)?;

        let mut value_parameters: Option<SmallVec<[(u8, Type); 4]>> = None;

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

            if let Some(value_parameters) = value_parameters.as_mut() {
                value_parameters.push((identifier_index, r#type));
            } else {
                value_parameters = Some(smallvec![(identifier_index, r#type)]);
            };

            function_compiler.minimum_register += 1;

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

        function_compiler.return_type = Some((return_type).clone());

        function_compiler.expect(Token::LeftBrace)?;
        function_compiler.compile()?;
        function_compiler.expect(Token::RightBrace)?;

        self.previous_token = function_compiler.previous_token;
        self.previous_position = function_compiler.previous_position;
        self.current_token = function_compiler.current_token;
        self.current_position = function_compiler.current_position;

        self.lexer.skip_to(self.current_position.1);

        let function_end = function_compiler.previous_position.1;
        let chunk = function_compiler.finish(None, value_parameters.clone());
        let function = ConcreteValue::function(chunk);
        let constant_index = self.push_or_get_constant(function);
        let destination = self.next_register();
        let function_type = FunctionType {
            type_parameters: None,
            value_parameters,
            return_type,
        };

        if let Some((identifier, _)) = identifier_info {
            self.declare_local(
                identifier,
                destination,
                Type::function(function_type.clone()),
                false,
                self.current_scope,
            );

            let load_constant = Instruction::load_constant(destination, constant_index, false);

            self.emit_instruction(
                load_constant,
                Type::function(function_type),
                Span(function_start, function_end),
            );
        } else {
            let load_constant = Instruction::from(LoadConstant {
                destination,
                constant_index,
                jump_next: false,
            });

            self.emit_instruction(
                load_constant,
                Type::function(function_type),
                Span(function_start, function_end),
            );
        }

        Ok(())
    }

    fn parse_call(&mut self) -> Result<(), CompileError> {
        let (last_instruction, last_instruction_type, _) =
            self.instructions
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

        let argument =
            last_instruction
                .as_argument()
                .ok_or_else(|| CompileError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;
        let function_return_type = match last_instruction_type {
            Type::Function(function_type) => function_type.return_type.clone(),
            Type::SelfChunk => self.return_type.clone().unwrap_or(Type::None),
            _ => {
                return Err(CompileError::ExpectedFunction {
                    found: self.previous_token.to_owned(),
                    actual_type: last_instruction_type.clone(),
                    position: self.previous_position,
                });
            }
        };
        let start = self.current_position.0;

        self.advance()?;

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
        let call = Instruction::from(Call {
            destination,
            function: argument,
            argument_count,
        });

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

    fn expect_addable_type(argument_type: &Type, position: &Span) -> Result<(), CompileError> {
        if matches!(
            argument_type,
            Type::Byte | Type::Character | Type::Float | Type::Integer | Type::String
        ) {
            Ok(())
        } else {
            Err(CompileError::CannotAddType {
                argument_type: argument_type.clone(),
                position: *position,
            })
        }
    }

    fn expect_addable_types(
        left: &Type,
        left_position: &Span,
        right: &Type,
        right_position: &Span,
    ) -> Result<(), CompileError> {
        if matches!(
            (left, right),
            (Type::Byte, Type::Byte)
                | (Type::Character, Type::String)
                | (Type::Character, Type::Character)
                | (Type::Float, Type::Float)
                | (Type::Integer, Type::Integer)
                | (Type::String, Type::Character)
                | (Type::String, Type::String),
        ) {
            Ok(())
        } else {
            Err(CompileError::CannotAddArguments {
                left_type: left.clone(),
                right_type: right.clone(),
                position: Span(left_position.0, right_position.1),
            })
        }
    }

    fn expect_dividable_type(argument_type: &Type, position: &Span) -> Result<(), CompileError> {
        if matches!(argument_type, Type::Byte | Type::Float | Type::Integer) {
            Ok(())
        } else {
            Err(CompileError::CannotDivideType {
                argument_type: argument_type.clone(),
                position: *position,
            })
        }
    }

    fn expect_dividable_types(
        left: &Type,
        left_position: &Span,
        right: &Type,
        right_position: &Span,
    ) -> Result<(), CompileError> {
        if matches!(
            (left, right),
            (Type::Byte, Type::Byte) | (Type::Float, Type::Float) | (Type::Integer, Type::Integer)
        ) {
            Ok(())
        } else {
            Err(CompileError::CannotDivideArguments {
                left_type: left.clone(),
                right_type: right.clone(),
                position: Span(left_position.0, right_position.1),
            })
        }
    }

    fn expect_modulable_type(argument_type: &Type, position: &Span) -> Result<(), CompileError> {
        if matches!(argument_type, Type::Byte | Type::Integer | Type::Float) {
            Ok(())
        } else {
            Err(CompileError::CannotModuloType {
                argument_type: argument_type.clone(),
                position: *position,
            })
        }
    }

    fn expect_modulable_types(
        left: &Type,
        left_position: &Span,
        right: &Type,
        right_position: &Span,
    ) -> Result<(), CompileError> {
        if matches!(
            (left, right),
            (Type::Byte, Type::Byte) | (Type::Integer, Type::Integer) | (Type::Float, Type::Float)
        ) {
            Ok(())
        } else {
            Err(CompileError::CannotModuloArguments {
                left_type: left.clone(),
                right_type: right.clone(),
                position: Span(left_position.0, right_position.1),
            })
        }
    }

    fn expect_multipliable_type(argument_type: &Type, position: &Span) -> Result<(), CompileError> {
        if matches!(argument_type, Type::Byte | Type::Float | Type::Integer) {
            Ok(())
        } else {
            Err(CompileError::CannotMultiplyType {
                argument_type: argument_type.clone(),
                position: *position,
            })
        }
    }

    fn expect_multipliable_types(
        left: &Type,
        left_position: &Span,
        right: &Type,
        right_position: &Span,
    ) -> Result<(), CompileError> {
        if matches!(
            (left, right),
            (Type::Byte, Type::Byte) | (Type::Float, Type::Float) | (Type::Integer, Type::Integer)
        ) {
            Ok(())
        } else {
            Err(CompileError::CannotMultiplyArguments {
                left_type: left.clone(),
                right_type: right.clone(),
                position: Span(left_position.0, right_position.1),
            })
        }
    }

    fn expect_subtractable_type(argument_type: &Type, position: &Span) -> Result<(), CompileError> {
        if matches!(argument_type, Type::Byte | Type::Float | Type::Integer) {
            Ok(())
        } else {
            Err(CompileError::CannotSubtractType {
                argument_type: argument_type.clone(),
                position: *position,
            })
        }
    }

    fn expect_subtractable_types(
        left: &Type,
        left_position: &Span,
        right: &Type,
        right_position: &Span,
    ) -> Result<(), CompileError> {
        if matches!(
            (left, right),
            (Type::Byte, Type::Byte) | (Type::Float, Type::Float) | (Type::Integer, Type::Integer)
        ) {
            Ok(())
        } else {
            Err(CompileError::CannotSubtractArguments {
                left_type: left.clone(),
                right_type: right.clone(),
                position: Span(left_position.0, right_position.1),
            })
        }
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
    ExpectedFunctionType {
        found: Type,
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
    CannotAddType {
        argument_type: Type,
        position: Span,
    },
    CannotAddArguments {
        left_type: Type,
        right_type: Type,
        position: Span,
    },
    CannotDivideType {
        argument_type: Type,
        position: Span,
    },
    CannotDivideArguments {
        left_type: Type,
        right_type: Type,
        position: Span,
    },
    CannotModuloType {
        argument_type: Type,
        position: Span,
    },
    CannotModuloArguments {
        left_type: Type,
        right_type: Type,
        position: Span,
    },
    CannotMultiplyType {
        argument_type: Type,
        position: Span,
    },
    CannotMultiplyArguments {
        left_type: Type,
        right_type: Type,
        position: Span,
    },
    CannotSubtractType {
        argument_type: Type,
        position: Span,
    },
    CannotSubtractArguments {
        left_type: Type,
        right_type: Type,
        position: Span,
    },
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
    ReturnTypeConflict {
        conflict: TypeConflict,
        position: Span,
    },

    // Chunk errors
    ConstantIndexOutOfBounds {
        index: usize,
        position: Span,
    },
    InstructionIndexOutOfBounds {
        index: usize,
        position: Span,
    },
    LocalIndexOutOfBounds {
        index: usize,
        position: Span,
    },

    // Wrappers around foreign errors
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
            Self::CannotAddArguments { .. } => "Cannot add these types",
            Self::CannotAddType { .. } => "Cannot add to this type",
            Self::CannotChainComparison { .. } => "Cannot chain comparison operations",
            Self::CannotDivideArguments { .. } => "Cannot divide these types",
            Self::CannotDivideType { .. } => "Cannot divide this type",
            Self::CannotModuloArguments { .. } => "Cannot modulo these types",
            Self::CannotModuloType { .. } => "Cannot modulo this type",
            Self::CannotMutateImmutableVariable { .. } => "Cannot mutate immutable variable",
            Self::CannotMultiplyArguments { .. } => "Cannot multiply these types",
            Self::CannotMultiplyType { .. } => "Cannot multiply this type",
            Self::CannotResolveRegisterType { .. } => "Cannot resolve register type",
            Self::CannotResolveVariableType { .. } => "Cannot resolve type",
            Self::CannotSubtractType { .. } => "Cannot subtract from this type",
            Self::CannotSubtractArguments { .. } => "Cannot subtract these types",
            Self::ConstantIndexOutOfBounds { .. } => "Constant index out of bounds",
            Self::ExpectedExpression { .. } => "Expected an expression",
            Self::ExpectedFunction { .. } => "Expected a function",
            Self::ExpectedFunctionType { .. } => "Expected a function type",
            Self::ExpectedMutableVariable { .. } => "Expected a mutable variable",
            Self::ExpectedToken { .. } => "Expected a specific token",
            Self::ExpectedTokenMultiple { .. } => "Expected one of multiple tokens",
            Self::IfElseBranchMismatch { .. } => "Type mismatch in if/else branches",
            Self::IfMissingElse { .. } => "If statement missing else branch",
            Self::InstructionIndexOutOfBounds { .. } => "Instruction index out of bounds",
            Self::InvalidAssignmentTarget { .. } => "Invalid assignment target",
            Self::Lex(error) => error.description(),
            Self::ListItemTypeConflict { .. } => "List item type conflict",
            Self::LocalIndexOutOfBounds { .. } => "Local index out of bounds",
            Self::ParseFloatError { .. } => "Failed to parse float",
            Self::ParseIntError { .. } => "Failed to parse integer",
            Self::ReturnTypeConflict { .. } => "Return type conflict",
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
            Self::ExpectedExpression { found, .. } => Some(format!("Found {found}")),
            Self::ExpectedFunction { found, actual_type, .. } => {
                Some(format!("Expected \"{found}\" to be a function but it has type {actual_type}"))
            }
            Self::ExpectedFunctionType { found, .. } => {
                Some(format!("Expected a function type but found {found}"))
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
            Self::ReturnTypeConflict {
                conflict: TypeConflict { expected, actual },
                ..
            } => Some(format!(
                "Expected return type \"{expected}\" but found \"{actual}\""
            )),
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
            Self::CannotAddArguments { position, .. } => *position,
            Self::CannotAddType { position, .. } => *position,
            Self::CannotChainComparison { position } => *position,
            Self::CannotDivideArguments { position, .. } => *position,
            Self::CannotDivideType { position, .. } => *position,
            Self::CannotModuloArguments { position, .. } => *position,
            Self::CannotModuloType { position, .. } => *position,
            Self::CannotMutateImmutableVariable { position, .. } => *position,
            Self::CannotMultiplyArguments { position, .. } => *position,
            Self::CannotMultiplyType { position, .. } => *position,
            Self::CannotResolveRegisterType { position, .. } => *position,
            Self::CannotResolveVariableType { position, .. } => *position,
            Self::CannotSubtractArguments { position, .. } => *position,
            Self::CannotSubtractType { position, .. } => *position,
            Self::ConstantIndexOutOfBounds { position, .. } => *position,
            Self::ExpectedExpression { position, .. } => *position,
            Self::ExpectedFunction { position, .. } => *position,
            Self::ExpectedFunctionType { position, .. } => *position,
            Self::ExpectedMutableVariable { position, .. } => *position,
            Self::ExpectedToken { position, .. } => *position,
            Self::ExpectedTokenMultiple { position, .. } => *position,
            Self::IfElseBranchMismatch { position, .. } => *position,
            Self::IfMissingElse { position } => *position,
            Self::InstructionIndexOutOfBounds { position, .. } => *position,
            Self::InvalidAssignmentTarget { position, .. } => *position,
            Self::Lex(error) => error.position(),
            Self::ListItemTypeConflict { position, .. } => *position,
            Self::LocalIndexOutOfBounds { position, .. } => *position,
            Self::ParseFloatError { position, .. } => *position,
            Self::ParseIntError { position, .. } => *position,
            Self::ReturnTypeConflict { position, .. } => *position,
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
