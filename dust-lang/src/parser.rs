use std::{
    fmt::{self, Display, Formatter},
    mem::replace,
    num::{ParseFloatError, ParseIntError},
    vec,
};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{
    AnnotatedError, Chunk, ChunkError, DustError, FunctionType, Identifier, Instruction, LexError,
    Lexer, NativeFunction, Operation, Span, Token, TokenKind, TokenOwned, Type, Value,
};

pub fn parse(source: &str) -> Result<Chunk, DustError> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer).map_err(|error| DustError::Parse { error, source })?;

    while !parser.is_eof() {
        parser
            .parse_statement(Allowed {
                assignment: true,
                explicit_return: false,
                implicit_return: true,
            })
            .map_err(|error| DustError::Parse { error, source })?;
    }

    Ok(parser.finish())
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize)]
pub struct Parser<'src> {
    lexer: Lexer<'src>,
    chunk: Chunk,

    current_statement_length: usize,
    current_is_expression: bool,
    minimum_register: u8,

    current_token: Token<'src>,
    current_position: Span,

    previous_token: Token<'src>,
    previous_position: Span,
}

impl<'src> Parser<'src> {
    pub fn new(mut lexer: Lexer<'src>) -> Result<Self, ParseError> {
        let (current_token, current_position) = lexer.next_token()?;

        log::info!(
            "Begin chunk with {} at {}",
            current_token.to_string().bold(),
            current_position.to_string()
        );

        Ok(Parser {
            lexer,
            chunk: Chunk::new(None),
            current_statement_length: 0,
            current_is_expression: false,
            minimum_register: 0,
            current_token,
            current_position,
            previous_token: Token::Eof,
            previous_position: Span(0, 0),
        })
    }

    pub fn finish(self) -> Chunk {
        log::info!("End chunk");

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
                    Some(instruction.a() + 1)
                } else {
                    None
                }
            })
            .unwrap_or(self.minimum_register)
    }

    fn advance(&mut self) -> Result<(), ParseError> {
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

    fn allow(&mut self, allowed: Token) -> Result<bool, ParseError> {
        if self.current_token == allowed {
            self.advance()?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if self.current_token == expected {
            self.advance()
        } else {
            Err(ParseError::ExpectedToken {
                expected: expected.kind(),
                found: self.current_token.to_owned(),
                position: self.current_position,
            })
        }
    }

    fn emit_instruction(&mut self, instruction: Instruction, position: Span) {
        log::debug!(
            "Emitting {} at {}",
            instruction.operation().to_string().bold(),
            position.to_string()
        );

        self.current_statement_length += 1;

        self.chunk.push_instruction(instruction, position);
    }

    fn optimize_statement(&mut self) {
        if let Some(
            [Operation::LoadBoolean | Operation::LoadConstant, Operation::LoadBoolean | Operation::LoadConstant, Operation::Jump, Operation::Equal | Operation::Less | Operation::LessEqual],
        ) = self.get_end_of_statement()
        {
            log::trace!("Optimizing boolean comparison");

            let mut instructions = self
                .chunk
                .instructions_mut()
                .iter_mut()
                .rev()
                .map(|(instruction, _)| instruction);
            let second_loader = instructions.next().unwrap();
            let first_loader = instructions.next().unwrap();

            first_loader.set_c_to_boolean(true);

            let mut second_loader_new = Instruction::with_operation(second_loader.operation());

            second_loader_new.set_a(first_loader.a());
            second_loader_new.set_b(second_loader.b());
            second_loader_new.set_c(second_loader.c());
            second_loader_new.set_b_to_boolean(second_loader.b_is_constant());
            second_loader_new.set_c_to_boolean(second_loader.c_is_constant());

            *second_loader = second_loader_new;
        }

        self.current_statement_length = 0;
    }

    fn get_last_value_operation(&self) -> Option<Operation> {
        self.chunk
            .instructions()
            .iter()
            .rev()
            .take(self.current_statement_length)
            .find_map(|(instruction, _)| {
                if instruction.yields_value() {
                    Some(instruction.operation())
                } else {
                    None
                }
            })
    }

    fn get_end_of_statement<const COUNT: usize>(&self) -> Option<[Operation; COUNT]> {
        if self.current_statement_length < COUNT {
            return None;
        }

        let mut operations = [Operation::Return; COUNT];

        for (index, (instruction, _)) in self
            .chunk
            .instructions()
            .iter()
            .rev()
            .take(COUNT)
            .enumerate()
        {
            operations[index] = instruction.operation();
        }

        Some(operations)
    }

    fn get_last_jump_mut(&mut self) -> Option<&mut Instruction> {
        self.chunk
            .instructions_mut()
            .iter_mut()
            .find_map(|(instruction, _)| {
                if let Operation::Jump = instruction.operation() {
                    Some(instruction)
                } else {
                    None
                }
            })
    }

    fn get_last_jumpable_mut(&mut self) -> Option<&mut Instruction> {
        self.chunk
            .instructions_mut()
            .iter_mut()
            .find_map(|(instruction, _)| {
                if let Operation::LoadBoolean | Operation::LoadConstant = instruction.operation() {
                    Some(instruction)
                } else {
                    None
                }
            })
    }

    fn emit_constant(&mut self, value: Value, position: Span) -> Result<(), ParseError> {
        let constant_index = self.chunk.push_constant(value, position)?;
        let register = self.next_register();

        self.emit_instruction(
            Instruction::load_constant(register, constant_index, false),
            position,
        );

        Ok(())
    }

    fn parse_boolean(&mut self, _: Allowed) -> Result<(), ParseError> {
        let position = self.current_position;

        if let Token::Boolean(text) = self.current_token {
            self.advance()?;

            let boolean = text.parse::<bool>().unwrap();
            let register = self.next_register();

            self.emit_instruction(
                Instruction::load_boolean(register, boolean, false),
                position,
            );

            self.current_is_expression = true;

            Ok(())
        } else {
            Err(ParseError::ExpectedToken {
                expected: TokenKind::Boolean,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_byte(&mut self, _: Allowed) -> Result<(), ParseError> {
        let position = self.current_position;

        if let Token::Byte(text) = self.current_token {
            self.advance()?;

            let byte = u8::from_str_radix(&text[2..], 16)
                .map_err(|error| ParseError::ParseIntError { error, position })?;
            let value = Value::byte(byte);

            self.emit_constant(value, position)?;

            self.current_is_expression = true;

            Ok(())
        } else {
            Err(ParseError::ExpectedToken {
                expected: TokenKind::Byte,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_character(&mut self, _: Allowed) -> Result<(), ParseError> {
        let position = self.current_position;

        if let Token::Character(character) = self.current_token {
            self.advance()?;

            let value = Value::character(character);

            self.emit_constant(value, position)?;

            self.current_is_expression = true;

            Ok(())
        } else {
            Err(ParseError::ExpectedToken {
                expected: TokenKind::Character,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_float(&mut self, _: Allowed) -> Result<(), ParseError> {
        let position = self.current_position;

        if let Token::Float(text) = self.current_token {
            self.advance()?;

            let float = text
                .parse::<f64>()
                .map_err(|error| ParseError::ParseFloatError {
                    error,
                    position: self.previous_position,
                })?;
            let value = Value::float(float);

            self.emit_constant(value, position)?;

            self.current_is_expression = true;

            Ok(())
        } else {
            Err(ParseError::ExpectedToken {
                expected: TokenKind::Float,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_integer(&mut self, _: Allowed) -> Result<(), ParseError> {
        let position = self.current_position;

        if let Token::Integer(text) = self.current_token {
            self.advance()?;

            let integer = text
                .parse::<i64>()
                .map_err(|error| ParseError::ParseIntError {
                    error,
                    position: self.previous_position,
                })?;
            let value = Value::integer(integer);

            self.emit_constant(value, position)?;

            self.current_is_expression = true;

            Ok(())
        } else {
            Err(ParseError::ExpectedToken {
                expected: TokenKind::Integer,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_string(&mut self, _: Allowed) -> Result<(), ParseError> {
        let position = self.current_position;

        if let Token::String(text) = self.current_token {
            self.advance()?;

            let value = Value::string(text);

            self.emit_constant(value, position)?;

            self.current_is_expression = true;

            Ok(())
        } else {
            Err(ParseError::ExpectedToken {
                expected: TokenKind::String,
                found: self.current_token.to_owned(),
                position,
            })
        }
    }

    fn parse_grouped(&mut self, _: Allowed) -> Result<(), ParseError> {
        self.allow(Token::LeftParenthesis)?;
        self.parse_expression()?;
        self.expect(Token::RightParenthesis)?;

        self.current_is_expression = true;

        Ok(())
    }

    fn parse_unary(&mut self, _: Allowed) -> Result<(), ParseError> {
        let operator = self.current_token;
        let operator_position = self.current_position;

        self.advance()?;
        self.parse_expression()?;

        let (previous_instruction, previous_position) = self
            .chunk
            .instructions_mut()
            .pop()
            .ok_or_else(|| ParseError::ExpectedExpression {
                found: self.previous_token.to_owned(),
                position: self.previous_position,
            })?;

        let (push_back, is_constant, argument) = {
            match previous_instruction.operation() {
                Operation::GetLocal => (false, false, previous_instruction.a()),
                Operation::LoadConstant => (false, true, previous_instruction.a()),
                Operation::LoadBoolean => (true, false, previous_instruction.a()),
                Operation::Close => {
                    return Err(ParseError::ExpectedExpression {
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
                return Err(ParseError::ExpectedTokenMultiple {
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

        self.current_is_expression = true;

        Ok(())
    }

    fn handle_binary_argument(
        &mut self,
        instruction: &Instruction,
    ) -> Result<(bool, bool, bool, u8), ParseError> {
        let mut push_back = false;
        let mut is_constant = false;
        let mut is_mutable_local = false;
        let argument = match instruction.operation() {
            Operation::GetLocal => {
                let local_index = instruction.b();
                let local = self.chunk.get_local(local_index, self.current_position)?;
                is_mutable_local = local.is_mutable;

                local.register_index
            }
            Operation::LoadConstant => {
                is_constant = true;

                instruction.b()
            }
            Operation::LoadBoolean => instruction.a(),
            Operation::Close => {
                return Err(ParseError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                });
            }
            _ => {
                push_back = true;

                self.next_register()
            }
        };

        Ok((push_back, is_constant, is_mutable_local, argument))
    }

    fn parse_math_binary(&mut self) -> Result<(), ParseError> {
        let (left_instruction, left_position) =
            self.chunk
                .instructions_mut()
                .pop()
                .ok_or_else(|| ParseError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
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
                return Err(ParseError::ExpectedMutableVariable {
                    found: self.previous_token.to_owned(),
                    position: left_position,
                });
            }
        }

        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        let (right_instruction, right_position) =
            self.chunk
                .instructions_mut()
                .pop()
                .ok_or_else(|| ParseError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;
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
                return Err(ParseError::ExpectedTokenMultiple {
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
            self.current_is_expression = false;
        } else {
            self.current_is_expression = true;
        }

        Ok(())
    }

    fn parse_comparison_binary(&mut self) -> Result<(), ParseError> {
        if let Some(Operation::Equal | Operation::Less | Operation::LessEqual) =
            self.get_last_value_operation()
        {
            return Err(ParseError::CannotChainComparison {
                position: self.current_position,
            });
        }

        let (left_instruction, left_position) =
            self.chunk
                .instructions_mut()
                .pop()
                .ok_or_else(|| ParseError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;
        let (push_back_left, left_is_constant, _, left) =
            self.handle_binary_argument(&left_instruction)?;

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(&operator);

        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        let (right_instruction, right_position) =
            self.chunk
                .instructions_mut()
                .pop()
                .ok_or_else(|| ParseError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
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
                return Err(ParseError::ExpectedTokenMultiple {
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

        self.current_is_expression = true;

        Ok(())
    }

    fn parse_logical_binary(&mut self) -> Result<(), ParseError> {
        let start_length = self.chunk.len();
        let (left_instruction, left_position) =
            self.chunk
                .instructions_mut()
                .pop()
                .ok_or_else(|| ParseError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                })?;

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(&operator);

        let test_instruction = match operator {
            Token::DoubleAmpersand => Instruction::test(left_instruction.a(), false),
            Token::DoublePipe => Instruction::test(left_instruction.a(), true),
            _ => {
                return Err(ParseError::ExpectedTokenMultiple {
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

        self.current_is_expression = true;

        Ok(())
    }

    fn parse_variable(&mut self, allowed: Allowed) -> Result<(), ParseError> {
        let token = self.current_token;
        let start_position = self.current_position;

        let local_index = if let Token::Identifier(text) = token {
            if let Ok(local_index) = self.chunk.get_local_index(text, start_position) {
                local_index
            } else if let Some(name) = self.chunk.name() {
                if name.as_str() == text {
                    let register = self.next_register();

                    self.emit_instruction(Instruction::load_self(register), start_position);

                    self.chunk.declare_local(
                        Identifier::new(text),
                        None,
                        false,
                        register,
                        start_position,
                    )?;

                    self.current_is_expression = true;

                    return Ok(());
                }

                return if NativeFunction::from_str(text).is_some() {
                    self.parse_native_call(allowed)
                } else {
                    Err(ParseError::UndeclaredVariable {
                        identifier: Identifier::new(text),
                        position: start_position,
                    })
                };
            } else {
                return if NativeFunction::from_str(text).is_some() {
                    self.parse_native_call(allowed)
                } else {
                    Err(ParseError::UndeclaredVariable {
                        identifier: Identifier::new(text),
                        position: start_position,
                    })
                };
            }
        } else {
            return Err(ParseError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: self.current_token.to_owned(),
                position: start_position,
            });
        };

        self.advance()?;

        let is_mutable = self
            .chunk
            .get_local(local_index, start_position)?
            .is_mutable;

        if self.allow(Token::Equal)? {
            if !allowed.assignment {
                return Err(ParseError::InvalidAssignmentTarget {
                    found: self.current_token.to_owned(),
                    position: self.current_position,
                });
            }

            if !is_mutable {
                return Err(ParseError::CannotMutateImmutableVariable {
                    identifier: self.chunk.get_identifier(local_index).cloned().unwrap(),
                    position: start_position,
                });
            }

            self.parse_expression()?;

            let (mut previous_instruction, previous_position) =
                self.chunk.instructions_mut().pop().ok_or_else(|| {
                    ParseError::ExpectedExpression {
                        found: self.previous_token.to_owned(),
                        position: self.previous_position,
                    }
                })?;

            if previous_instruction.operation().is_math() {
                let register_index = self
                    .chunk
                    .get_local(local_index, start_position)?
                    .register_index;

                log::trace!("Condensing SET_LOCAL to binary math expression");

                previous_instruction.set_a(register_index);
                self.emit_instruction(previous_instruction, self.current_position);

                return Ok(());
            }

            let register = self.next_register();

            self.emit_instruction(previous_instruction, previous_position);
            self.emit_instruction(
                Instruction::set_local(register, local_index),
                start_position,
            );
            self.optimize_statement();

            self.current_is_expression = false;
        } else {
            let register = self.next_register();

            self.emit_instruction(
                Instruction::get_local(register, local_index),
                self.previous_position,
            );

            self.current_is_expression = true;
        }

        Ok(())
    }

    fn parse_type_from(&mut self, token: Token, position: Span) -> Result<Type, ParseError> {
        match token {
            Token::Bool => Ok(Type::Boolean),
            Token::FloatKeyword => Ok(Type::Float),
            Token::Int => Ok(Type::Integer),
            Token::Str => Ok(Type::String { length: None }),
            _ => Err(ParseError::ExpectedTokenMultiple {
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

    fn parse_block(&mut self, allowed: Allowed) -> Result<(), ParseError> {
        self.advance()?;
        self.chunk.begin_scope();

        while !self.allow(Token::RightCurlyBrace)? && !self.is_eof() {
            self.parse_statement(allowed)?;
        }

        self.chunk.end_scope();

        Ok(())
    }

    fn parse_list(&mut self, _: Allowed) -> Result<(), ParseError> {
        let start = self.current_position.0;

        self.advance()?;

        let start_register = self.next_register();

        while !self.allow(Token::RightSquareBrace)? && !self.is_eof() {
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

        let to_register = self.next_register();
        let end_register = to_register.saturating_sub(1);
        let end = self.current_position.1;

        self.emit_instruction(
            Instruction::load_list(to_register, start_register, end_register),
            Span(start, end),
        );

        self.current_is_expression = true;

        Ok(())
    }

    fn parse_if(&mut self, allowed: Allowed) -> Result<(), ParseError> {
        self.advance()?;
        self.parse_expression()?;

        if let Some(
            [Operation::LoadBoolean, Operation::LoadBoolean, Operation::Jump, Operation::Equal | Operation::Less | Operation::LessEqual],
        ) = self.get_end_of_statement()
        {
            self.chunk.instructions_mut().pop();
            self.chunk.instructions_mut().pop();
        }

        let block_allowed = Allowed {
            assignment: allowed.assignment,
            explicit_return: allowed.explicit_return,
            implicit_return: false,
        };

        if let Token::LeftCurlyBrace = self.current_token {
            let block_start = self.chunk.len();

            self.parse_block(block_allowed)?;

            let block_end = self.chunk.len();
            let jump_distance = (block_end - block_start) as u8;

            if let Some(jump) = self.get_last_jump_mut() {
                jump.set_b(jump_distance);
            }
        } else {
            return Err(ParseError::ExpectedToken {
                expected: TokenKind::LeftCurlyBrace,
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        }

        let if_block_is_expression = self
            .chunk
            .instructions()
            .iter()
            .find_map(|(instruction, _)| {
                if !matches!(instruction.operation(), Operation::Jump) {
                    Some(true)
                } else {
                    None
                }
            })
            .unwrap_or(false);

        if let Token::Else = self.current_token {
            let else_start = self.chunk.len();

            self.parse_else(allowed, block_allowed)?;

            let else_end = self.chunk.len();
            let jump_distance = (else_end - else_start) as u8;
            self.current_is_expression = if_block_is_expression
                && self
                    .chunk
                    .instructions()
                    .iter()
                    .find_map(|(instruction, _)| {
                        if !matches!(instruction.operation(), Operation::Jump) {
                            Some(true)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(false);

            if jump_distance == 1 {
                if let Some(skippable) = self.get_last_jumpable_mut() {
                    skippable.set_c_to_boolean(true);
                } else {
                    self.chunk.insert_instruction(
                        else_start,
                        Instruction::jump(jump_distance, true),
                        self.current_position,
                    )?;
                }
            } else {
                self.chunk.insert_instruction(
                    else_start,
                    Instruction::jump(jump_distance, true),
                    self.current_position,
                )?;
            }
        } else {
            self.current_is_expression = false;
        }

        Ok(())
    }

    fn parse_else(&mut self, allowed: Allowed, block_allowed: Allowed) -> Result<(), ParseError> {
        self.advance()?;

        let if_block_end = self.chunk.len();

        if let Token::If = self.current_token {
            self.parse_if(allowed)?;
        } else if let Token::LeftCurlyBrace = self.current_token {
            self.parse_block(block_allowed)?;

            let else_end = self.chunk.len();

            if else_end - if_block_end > 1 {
                let jump_distance = (else_end - if_block_end) as u8;

                self.chunk.insert_instruction(
                    if_block_end,
                    Instruction::jump(jump_distance, true),
                    self.current_position,
                )?;
            }

            self.current_is_expression = self
                .chunk
                .instructions()
                .last()
                .map_or(false, |(instruction, _)| instruction.yields_value());
        } else {
            return Err(ParseError::ExpectedTokenMultiple {
                expected: &[TokenKind::If, TokenKind::LeftCurlyBrace],
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        }

        self.current_is_expression = self
            .chunk
            .instructions()
            .last()
            .map_or(false, |(instruction, _)| instruction.yields_value());

        self.optimize_statement();

        Ok(())
    }

    fn parse_while(&mut self, allowed: Allowed) -> Result<(), ParseError> {
        self.advance()?;

        let expression_start = self.chunk.len() as u8;

        self.parse_expression()?;

        if let Some(
            [Operation::LoadBoolean, Operation::LoadBoolean, Operation::Jump, Operation::Equal | Operation::Less | Operation::LessEqual],
        ) = self.get_end_of_statement()
        {
            self.chunk.instructions_mut().pop();
            self.chunk.instructions_mut().pop();
            self.chunk.instructions_mut().pop();
        }

        let block_start = self.chunk.len();

        self.parse_block(Allowed {
            assignment: true,
            explicit_return: allowed.explicit_return,
            implicit_return: false,
        })?;

        let block_end = self.chunk.len() as u8;

        self.chunk.insert_instruction(
            block_start,
            Instruction::jump(block_end - block_start as u8 + 1, true),
            self.current_position,
        )?;

        let jump_back_distance = block_end - expression_start + 1;
        let jump_back = Instruction::jump(jump_back_distance, false);

        self.emit_instruction(jump_back, self.current_position);
        self.optimize_statement();

        self.current_is_expression = false;

        Ok(())
    }

    fn parse_native_call(&mut self, _: Allowed) -> Result<(), ParseError> {
        let native_function = if let Token::Identifier(text) = self.current_token {
            NativeFunction::from_str(text).unwrap()
        } else {
            return Err(ParseError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        };
        let start = self.current_position.0;
        let start_register = self.next_register();

        self.advance()?;

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
        self.current_is_expression = native_function.returns_value();

        self.emit_instruction(
            Instruction::call_native(to_register, native_function, argument_count),
            Span(start, end),
        );
        Ok(())
    }

    fn parse_statement(&mut self, allowed: Allowed) -> Result<(), ParseError> {
        self.parse(Precedence::None, allowed)?;

        if allowed.implicit_return {
            self.parse_implicit_return()?;
        }

        Ok(())
    }

    fn parse_expression(&mut self) -> Result<(), ParseError> {
        self.parse(
            Precedence::None,
            Allowed {
                assignment: false,
                explicit_return: false,
                implicit_return: false,
            },
        )
    }

    fn parse_sub_expression(&mut self, precedence: &Precedence) -> Result<(), ParseError> {
        self.parse(
            precedence.increment(),
            Allowed {
                assignment: false,
                explicit_return: false,
                implicit_return: false,
            },
        )
    }

    fn parse_return(&mut self, allowed: Allowed) -> Result<(), ParseError> {
        if !allowed.explicit_return {
            return Err(ParseError::UnexpectedReturn {
                position: self.current_position,
            });
        }

        let start = self.current_position.0;

        self.advance()?;

        let has_return_value = if matches!(
            self.current_token,
            Token::Semicolon | Token::RightCurlyBrace
        ) {
            false
        } else {
            self.parse_expression()?;

            true
        };
        let end = self.current_position.1;

        self.emit_instruction(Instruction::r#return(has_return_value), Span(start, end));
        self.optimize_statement();

        self.current_is_expression = false;

        Ok(())
    }

    fn parse_implicit_return(&mut self) -> Result<(), ParseError> {
        if !self.current_is_expression {
            return Ok(());
        }

        let end_of_statement = matches!(
            self.current_token,
            Token::Eof | Token::RightCurlyBrace | Token::Semicolon
        );
        let has_semicolon = self.allow(Token::Semicolon)?;
        let returned = self
            .chunk
            .instructions()
            .last()
            .map(|(instruction, _)| matches!(instruction.operation(), Operation::Return))
            .unwrap_or(false);

        if end_of_statement && !has_semicolon && !returned {
            self.emit_instruction(Instruction::r#return(true), self.current_position);
        }

        Ok(())
    }

    fn parse_let_statement(&mut self, allowed: Allowed) -> Result<(), ParseError> {
        if !allowed.assignment {
            return Err(ParseError::ExpectedExpression {
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        }

        self.advance()?;

        let is_mutable = self.allow(Token::Mut)?;
        let position = self.current_position;
        let identifier = if let Token::Identifier(text) = self.current_token {
            self.advance()?;

            Identifier::new(text)
        } else {
            return Err(ParseError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: self.current_token.to_owned(),
                position,
            });
        };
        let r#type = if self.allow(Token::Colon)? {
            let r#type = self.parse_type_from(self.current_token, self.current_position)?;

            self.advance()?;

            Some(r#type)
        } else {
            None
        };
        let register = self.next_register();

        self.expect(Token::Equal)?;
        self.parse_expression()?;

        let local_index = self
            .chunk
            .declare_local(identifier, r#type, is_mutable, register, position)?;
        let register = self.next_register().saturating_sub(1);

        self.emit_instruction(
            Instruction::define_local(register, local_index, is_mutable),
            position,
        );
        self.optimize_statement();

        self.current_is_expression = false;

        Ok(())
    }

    fn parse_function(&mut self, _: Allowed) -> Result<(), ParseError> {
        let function_start = self.current_position.0;
        let mut function_parser = Parser::new(self.lexer)?;
        let identifier = if let Token::Identifier(text) = function_parser.current_token {
            let position = function_parser.current_position;
            let identifier = Identifier::new(text);

            function_parser.advance()?;
            function_parser.chunk.set_name(identifier.clone());

            Some((identifier, position))
        } else {
            None
        };

        function_parser.expect(Token::LeftParenthesis)?;

        let mut value_parameters: Option<Vec<(Identifier, Type)>> = None;

        while function_parser.current_token != Token::RightParenthesis {
            let start = function_parser.current_position.0;
            let is_mutable = function_parser.allow(Token::Mut)?;
            let parameter = if let Token::Identifier(text) = function_parser.current_token {
                function_parser.advance()?;

                Identifier::new(text)
            } else {
                return Err(ParseError::ExpectedToken {
                    expected: TokenKind::Identifier,
                    found: function_parser.current_token.to_owned(),
                    position: function_parser.current_position,
                });
            };

            function_parser.expect(Token::Colon)?;

            let r#type = function_parser.parse_type_from(
                function_parser.current_token,
                function_parser.current_position,
            )?;

            function_parser.advance()?;

            let end = function_parser.current_position.1;

            if let Some(value_parameters) = value_parameters.as_mut() {
                value_parameters.push((parameter.clone(), r#type.clone()));
            } else {
                value_parameters = Some(vec![(parameter.clone(), r#type.clone())]);
            };

            let register = value_parameters
                .as_ref()
                .map(|values| values.len() as u8 - 1)
                .unwrap_or(0);

            function_parser.chunk.declare_local(
                parameter,
                Some(r#type),
                is_mutable,
                register,
                Span(start, end),
            )?;

            function_parser.minimum_register += 1;

            function_parser.allow(Token::Comma)?;
        }

        function_parser.advance()?;

        let return_type = if function_parser.allow(Token::ArrowThin)? {
            let r#type = function_parser.parse_type_from(
                function_parser.current_token,
                function_parser.current_position,
            )?;

            function_parser.advance()?;

            Some(Box::new(r#type))
        } else {
            None
        };

        function_parser.expect(Token::LeftCurlyBrace)?;

        while function_parser.current_token != Token::RightCurlyBrace {
            function_parser.parse_statement(Allowed {
                assignment: true,
                explicit_return: true,
                implicit_return: true,
            })?;
        }

        function_parser.advance()?;

        self.previous_token = function_parser.previous_token;
        self.previous_position = function_parser.previous_position;
        self.current_token = function_parser.current_token;
        self.current_position = function_parser.current_position;

        let function_type = FunctionType {
            type_parameters: None,
            value_parameters,
            return_type,
        };
        let function = Value::function(function_parser.chunk, function_type.clone());
        let function_end = self.current_position.1;

        self.lexer.skip_to(function_end);

        if let Some((identifier, identifier_position)) = identifier {
            let register = self.next_register();
            let local_index = self.chunk.declare_local(
                identifier,
                Some(Type::Function(function_type)),
                false,
                register,
                Span(function_start, function_end),
            )?;

            self.emit_constant(function, Span(function_start, function_end))?;
            self.emit_instruction(
                Instruction::define_local(register, local_index, false),
                identifier_position,
            );
            self.optimize_statement();

            self.current_is_expression = false;
        } else {
            self.emit_constant(function, Span(function_start, function_end))?;
            self.optimize_statement();

            self.current_is_expression = true;
        }

        Ok(())
    }

    fn parse_call(&mut self) -> Result<(), ParseError> {
        let (last_instruction, _) = self.chunk.instructions().last().copied().ok_or_else(|| {
            ParseError::ExpectedExpression {
                found: self.previous_token.to_owned(),
                position: self.previous_position,
            }
        })?;

        if !last_instruction.yields_value() {
            return Err(ParseError::ExpectedExpression {
                found: self.previous_token.to_owned(),
                position: self.previous_position,
            });
        }

        let function_register = last_instruction.a();
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

        self.current_is_expression = true;

        Ok(())
    }

    fn parse_semicolon(&mut self, _: Allowed) -> Result<(), ParseError> {
        self.current_is_expression = false;

        self.optimize_statement();
        self.advance()
    }

    fn expect_expression(&mut self, _: Allowed) -> Result<(), ParseError> {
        if self.current_token.is_expression() {
            Ok(())
        } else {
            Err(ParseError::ExpectedExpression {
                found: self.current_token.to_owned(),
                position: self.current_position,
            })
        }
    }

    fn parse(&mut self, precedence: Precedence, allowed: Allowed) -> Result<(), ParseError> {
        if let Some(prefix_parser) = ParseRule::from(&self.current_token).prefix {
            log::debug!(
                "{} is prefix with precedence {precedence}",
                self.current_token.to_string().bold(),
            );

            prefix_parser(self, allowed)?;
        }

        let mut infix_rule = ParseRule::from(&self.current_token);

        while precedence <= infix_rule.precedence {
            if let Some(infix_parser) = infix_rule.infix {
                log::debug!(
                    "{} is infix with precedence {precedence}",
                    self.current_token.to_string().bold(),
                );

                if !allowed.assignment && self.current_token == Token::Equal {
                    return Err(ParseError::InvalidAssignmentTarget {
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Context {
    None,
    Assignment,
}

#[derive(Debug, Clone, Copy)]
struct Allowed {
    pub assignment: bool,
    pub explicit_return: bool,
    pub implicit_return: bool,
}

type PrefixFunction<'a> = fn(&mut Parser<'a>, Allowed) -> Result<(), ParseError>;
type InfixFunction<'a> = fn(&mut Parser<'a>) -> Result<(), ParseError>;

#[derive(Debug, Clone, Copy)]
struct ParseRule<'a> {
    pub prefix: Option<PrefixFunction<'a>>,
    pub infix: Option<InfixFunction<'a>>,
    pub precedence: Precedence,
}

impl From<&Token<'_>> for ParseRule<'_> {
    fn from(token: &Token) -> Self {
        match token {
            Token::ArrowThin => ParseRule {
                prefix: Some(Parser::expect_expression),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Async => todo!(),
            Token::Bang => ParseRule {
                prefix: Some(Parser::parse_unary),
                infix: None,
                precedence: Precedence::Unary,
            },
            Token::BangEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Equality,
            },
            Token::Bool => ParseRule {
                prefix: Some(Parser::expect_expression),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Boolean(_) => ParseRule {
                prefix: Some(Parser::parse_boolean),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Break => todo!(),
            Token::Byte(_) => ParseRule {
                prefix: Some(Parser::parse_byte),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Character(_) => ParseRule {
                prefix: Some(Parser::parse_character),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Colon => ParseRule {
                prefix: Some(Parser::expect_expression),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Comma => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::Dot => ParseRule {
                prefix: Some(Parser::expect_expression),
                infix: None,
                precedence: Precedence::None,
            },
            Token::DoubleAmpersand => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_logical_binary),
                precedence: Precedence::LogicalAnd,
            },
            Token::DoubleEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Equality,
            },
            Token::DoublePipe => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_logical_binary),
                precedence: Precedence::LogicalOr,
            },
            Token::DoubleDot => ParseRule {
                prefix: Some(Parser::expect_expression),
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
                prefix: Some(Parser::parse_float),
                infix: None,
                precedence: Precedence::None,
            },
            Token::FloatKeyword => ParseRule {
                prefix: Some(Parser::expect_expression),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Fn => ParseRule {
                prefix: Some(Parser::parse_function),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Greater => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            Token::GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            Token::Identifier(_) => ParseRule {
                prefix: Some(Parser::parse_variable),
                infix: None,
                precedence: Precedence::None,
            },
            Token::If => ParseRule {
                prefix: Some(Parser::parse_if),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Int => ParseRule {
                prefix: Some(Parser::expect_expression),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Integer(_) => ParseRule {
                prefix: Some(Parser::parse_integer),
                infix: None,
                precedence: Precedence::None,
            },
            Token::LeftCurlyBrace => ParseRule {
                prefix: Some(Parser::parse_block),
                infix: None,
                precedence: Precedence::None,
            },
            Token::LeftParenthesis => ParseRule {
                prefix: Some(Parser::parse_grouped),
                infix: Some(Parser::parse_call),
                precedence: Precedence::Call,
            },
            Token::LeftSquareBrace => ParseRule {
                prefix: Some(Parser::parse_list),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Less => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            Token::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            Token::Let => ParseRule {
                prefix: Some(Parser::parse_let_statement),
                infix: None,
                precedence: Precedence::Assignment,
            },
            Token::Loop => todo!(),
            Token::Map => todo!(),
            Token::Minus => ParseRule {
                prefix: Some(Parser::parse_unary),
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Term,
            },
            Token::MinusEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Assignment,
            },
            Token::Mut => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::Percent => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Factor,
            },
            Token::PercentEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Assignment,
            },
            Token::Plus => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Term,
            },
            Token::PlusEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Assignment,
            },
            Token::Return => ParseRule {
                prefix: Some(Parser::parse_return),
                infix: None,
                precedence: Precedence::None,
            },
            Token::RightCurlyBrace => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::RightParenthesis => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::RightSquareBrace => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::Semicolon => ParseRule {
                prefix: Some(Parser::parse_semicolon),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Slash => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Factor,
            },
            Token::SlashEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Assignment,
            },
            Token::Star => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Factor,
            },
            Token::StarEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Assignment,
            },
            Token::Str => ParseRule {
                prefix: Some(Parser::expect_expression),
                infix: None,
                precedence: Precedence::None,
            },
            Token::String(_) => ParseRule {
                prefix: Some(Parser::parse_string),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Struct => todo!(),
            Token::While => ParseRule {
                prefix: Some(Parser::parse_while),
                infix: None,
                precedence: Precedence::None,
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    CannotChainComparison {
        position: Span,
    },
    CannotMutateImmutableVariable {
        identifier: Identifier,
        position: Span,
    },
    ExpectedExpression {
        found: TokenOwned,
        position: Span,
    },
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
    ExpectedMutableVariable {
        found: TokenOwned,
        position: Span,
    },
    InvalidAssignmentTarget {
        found: TokenOwned,
        position: Span,
    },
    UndeclaredVariable {
        identifier: Identifier,
        position: Span,
    },
    UnexpectedReturn {
        position: Span,
    },
    RegisterOverflow {
        position: Span,
    },
    RegisterUnderflow {
        position: Span,
    },

    // Wrappers around foreign errors
    Chunk(ChunkError),
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

impl From<ChunkError> for ParseError {
    fn from(error: ChunkError) -> Self {
        Self::Chunk(error)
    }
}

impl AnnotatedError for ParseError {
    fn title() -> &'static str {
        "Parse Error"
    }

    fn description(&self) -> &'static str {
        match self {
            Self::CannotChainComparison { .. } => "Cannot chain comparison",
            Self::CannotMutateImmutableVariable { .. } => "Cannot mutate immutable variable",
            Self::ExpectedExpression { .. } => "Expected an expression",
            Self::ExpectedToken { .. } => "Expected a specific token",
            Self::ExpectedTokenMultiple { .. } => "Expected one of multiple tokens",
            Self::ExpectedMutableVariable { .. } => "Expected a mutable variable",
            Self::InvalidAssignmentTarget { .. } => "Invalid assignment target",
            Self::UndeclaredVariable { .. } => "Undeclared variable",
            Self::UnexpectedReturn { .. } => "Unexpected return",
            Self::RegisterOverflow { .. } => "Register overflow",
            Self::RegisterUnderflow { .. } => "Register underflow",
            Self::ParseFloatError { .. } => "Failed to parse float",
            Self::ParseIntError { .. } => "Failed to parse integer",
            Self::Chunk(error) => error.description(),
            Self::Lex(error) => error.description(),
        }
    }

    fn details(&self) -> Option<String> {
        match self {
            Self::CannotChainComparison { .. } => {
                Some("Cannot chain comparison operations".to_string())
            }
            Self::CannotMutateImmutableVariable { identifier, .. } => {
                Some(format!("Cannot mutate immutable variable {identifier}"))
            }
            Self::ExpectedExpression { found, .. } => Some(format!("Found {found}")),
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
            Self::ExpectedMutableVariable { found, .. } => {
                Some(format!("Expected mutable variable, found {found}"))
            }
            Self::InvalidAssignmentTarget { found, .. } => {
                Some(format!("Invalid assignment target, found {found}"))
            }
            Self::UndeclaredVariable { identifier, .. } => {
                Some(format!("Undeclared variable {identifier}"))
            }
            Self::UnexpectedReturn { .. } => None,
            Self::RegisterOverflow { .. } => None,
            Self::RegisterUnderflow { .. } => None,
            Self::ParseFloatError { error, .. } => Some(error.to_string()),
            Self::ParseIntError { error, .. } => Some(error.to_string()),
            Self::Chunk(error) => error.details(),
            Self::Lex(error) => error.details(),
        }
    }

    fn position(&self) -> Span {
        match self {
            Self::CannotChainComparison { position } => *position,
            Self::CannotMutateImmutableVariable { position, .. } => *position,
            Self::ExpectedExpression { position, .. } => *position,
            Self::ExpectedToken { position, .. } => *position,
            Self::ExpectedTokenMultiple { position, .. } => *position,
            Self::ExpectedMutableVariable { position, .. } => *position,
            Self::InvalidAssignmentTarget { position, .. } => *position,
            Self::UndeclaredVariable { position, .. } => *position,
            Self::UnexpectedReturn { position } => *position,
            Self::RegisterOverflow { position } => *position,
            Self::RegisterUnderflow { position } => *position,
            Self::ParseFloatError { position, .. } => *position,
            Self::ParseIntError { position, .. } => *position,
            Self::Chunk(error) => error.position(),
            Self::Lex(error) => error.position(),
        }
    }
}

impl From<LexError> for ParseError {
    fn from(error: LexError) -> Self {
        Self::Lex(error)
    }
}
