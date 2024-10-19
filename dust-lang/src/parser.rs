use std::{
    fmt::{self, Display, Formatter},
    mem::replace,
    num::{ParseFloatError, ParseIntError},
};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{
    AnnotatedError, Chunk, ChunkError, DustError, FunctionType, Identifier, Instruction, LexError,
    Lexer, Operation, Span, Token, TokenKind, TokenOwned, Type, Value,
};

pub fn parse(source: &str) -> Result<Chunk, DustError> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer).map_err(|error| DustError::Parse { error, source })?;

    while !parser.is_eof() {
        parser
            .parse_statement(
                Allowed {
                    assignment: true,
                    explicit_return: false,
                    implicit_return: true,
                },
                Context::None,
            )
            .map_err(|error| DustError::Parse { error, source })?;
    }

    Ok(parser.take_chunk())
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize)]
pub struct Parser<'src> {
    chunk: Chunk,
    lexer: Lexer<'src>,

    current_register: u8,

    current_token: Token<'src>,
    current_position: Span,

    previous_token: Token<'src>,
    previous_position: Span,

    parsed_expression: bool,
    latest_value_type: Option<Type>,
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
            chunk: Chunk::new(),
            current_register: 0,
            current_token,
            current_position,
            previous_token: Token::Eof,
            previous_position: Span(0, 0),
            parsed_expression: false,
            latest_value_type: None,
        })
    }

    pub fn take_chunk(self) -> Chunk {
        log::info!("End chunk");

        self.chunk
    }

    fn is_eof(&self) -> bool {
        matches!(self.current_token, Token::Eof)
    }

    fn increment_register(&mut self) -> Result<(), ParseError> {
        let current = self.current_register;

        if current == u8::MAX {
            Err(ParseError::RegisterOverflow {
                position: self.current_position,
            })
        } else {
            self.current_register += 1;

            Ok(())
        }
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

        self.chunk.push_instruction(instruction, position);
    }

    fn emit_constant(&mut self, value: Value, position: Span) -> Result<(), ParseError> {
        self.latest_value_type = Some(value.r#type());

        let constant_index = self.chunk.push_constant(value, position)?;

        self.emit_instruction(
            Instruction::load_constant(self.current_register, constant_index, false),
            position,
        );

        Ok(())
    }

    fn parse_boolean(&mut self, _: Allowed) -> Result<(), ParseError> {
        let position = self.current_position;

        if let Token::Boolean(text) = self.current_token {
            self.advance()?;

            let boolean = text.parse::<bool>().unwrap();

            self.emit_instruction(
                Instruction::load_boolean(self.current_register, boolean, false),
                position,
            );

            self.latest_value_type = Some(Type::Boolean);
            self.parsed_expression = true;

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

            self.parsed_expression = true;

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

            self.parsed_expression = true;

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

            self.parsed_expression = true;

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

            self.parsed_expression = true;

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

            self.parsed_expression = true;

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

        self.parsed_expression = true;

        Ok(())
    }

    fn parse_unary(&mut self, _: Allowed) -> Result<(), ParseError> {
        let operator = self.current_token;
        let operator_position = self.current_position;

        self.advance()?;
        self.parse_expression()?;

        let (previous_instruction, previous_position) =
            self.chunk.pop_instruction(self.current_position)?;

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
            self.increment_register()?;
        }

        let mut instruction = match operator.kind() {
            TokenKind::Bang => Instruction::not(self.current_register, argument),
            TokenKind::Minus => Instruction::negate(self.current_register, argument),
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

        if push_back {
            self.emit_instruction(previous_instruction, previous_position);
        }

        self.emit_instruction(instruction, operator_position);

        self.parsed_expression = true;

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

                instruction.a()
            }
        };

        Ok((push_back, is_constant, is_mutable_local, argument))
    }

    fn parse_math_binary(&mut self) -> Result<(), ParseError> {
        let (left_instruction, left_position) =
            self.chunk.pop_instruction(self.current_position)?;
        let (push_back_left, left_is_constant, left_is_mutable_local, left) =
            self.handle_binary_argument(&left_instruction)?;

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(&operator);

        if let TokenKind::PlusEqual
        | TokenKind::MinusEqual
        | TokenKind::StarEqual
        | TokenKind::SlashEqual = operator.kind()
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
            self.chunk.pop_instruction(self.current_position)?;
        let (push_back_right, right_is_constant, right_is_mutable_local, right) =
            self.handle_binary_argument(&right_instruction)?;

        let register = if left_is_mutable_local {
            left
        } else if right_is_mutable_local {
            right
        } else {
            let current = self.current_register;

            self.increment_register()?;

            current
        };

        let (mut new_instruction, is_expression) = match operator.kind() {
            TokenKind::Plus => (Instruction::add(register, left, right), true),
            TokenKind::PlusEqual => (Instruction::add(register, left, right), false),
            TokenKind::Minus => (Instruction::subtract(register, left, right), true),
            TokenKind::MinusEqual => (Instruction::subtract(register, left, right), false),
            TokenKind::Star => (Instruction::multiply(register, left, right), true),
            TokenKind::StarEqual => (Instruction::multiply(register, left, right), false),
            TokenKind::Slash => (Instruction::divide(register, left, right), true),
            TokenKind::SlashEqual => (Instruction::divide(register, left, right), false),
            TokenKind::Percent => (Instruction::modulo(register, left, right), true),
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
                    ],
                    found: operator.to_owned(),
                    position: operator_position,
                })
            }
        };

        self.parsed_expression = is_expression;

        if left_is_constant {
            new_instruction.set_b_is_constant();
        }

        if right_is_constant {
            new_instruction.set_c_is_constant();
        }

        let mut instructions = if !push_back_left && !push_back_right {
            self.emit_instruction(new_instruction, operator_position);

            return Ok(());
        } else if push_back_right && !push_back_left {
            vec![
                (right_instruction, right_position),
                (new_instruction, operator_position),
            ]
        } else if push_back_left && !push_back_right {
            vec![
                (left_instruction, left_position),
                (new_instruction, operator_position),
            ]
        } else {
            vec![
                (new_instruction, operator_position),
                (left_instruction, left_position),
                (right_instruction, right_position),
            ]
        };

        while let Ok(operation) = self.chunk.get_last_operation(operator_position) {
            if operation.is_math() {
                let (instruction, position) = self.chunk.pop_instruction(self.current_position)?;

                instructions.push((instruction, position));
            } else {
                break;
            }
        }

        instructions.sort_by_key(|(instruction, _)| instruction.a());

        for (instruction, position) in instructions {
            self.emit_instruction(instruction, position);
        }

        Ok(())
    }

    fn parse_comparison_binary(&mut self) -> Result<(), ParseError> {
        if let [Some(Operation::Jump), Some(Operation::Equal) | Some(Operation::Less) | Some(Operation::LessEqual)] =
            self.chunk.get_last_n_operations()
        {
            return Err(ParseError::CannotChainComparison {
                position: self.current_position,
            });
        }

        let (left_instruction, left_position) =
            self.chunk.pop_instruction(self.current_position)?;
        let (push_back_left, left_is_constant, _, left) =
            self.handle_binary_argument(&left_instruction)?;

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(&operator);

        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        let (right_instruction, right_position) =
            self.chunk.pop_instruction(self.current_position)?;
        let (push_back_right, right_is_constant, _, right) =
            self.handle_binary_argument(&right_instruction)?;

        let mut instruction = match operator {
            Token::DoubleEqual => Instruction::equal(true, left.saturating_sub(1), right),
            Token::BangEqual => Instruction::equal(false, left.saturating_sub(1), right),
            Token::Less => Instruction::less(true, left.saturating_sub(1), right),
            Token::LessEqual => Instruction::less_equal(true, left.saturating_sub(1), right),
            Token::Greater => Instruction::less_equal(false, left.saturating_sub(1), right),
            Token::GreaterEqual => Instruction::less(false, left.saturating_sub(1), right),

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

        self.parsed_expression = true;

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

        self.emit_instruction(instruction, operator_position);
        self.emit_instruction(Instruction::jump(1, true), operator_position);

        Ok(())
    }

    fn parse_logical_binary(&mut self) -> Result<(), ParseError> {
        let (left_instruction, left_position) =
            self.chunk.pop_instruction(self.current_position)?;

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(&operator);

        let instruction = match operator {
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

        self.increment_register()?;
        self.advance()?;
        self.parse_sub_expression(&rule.precedence)?;

        let (right_instruction, right_position) =
            self.chunk.pop_instruction(self.current_position)?;

        self.emit_instruction(left_instruction, left_position);
        self.emit_instruction(instruction, operator_position);
        self.emit_instruction(Instruction::jump(1, true), operator_position);
        self.emit_instruction(right_instruction, right_position);

        self.parsed_expression = true;

        Ok(())
    }

    fn parse_variable(&mut self, allowed: Allowed) -> Result<(), ParseError> {
        let token = self.current_token;
        let start_position = self.current_position;

        self.advance()?;

        let local_index = self.parse_identifier_from(token, start_position)?;
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
                self.chunk.pop_instruction(self.current_position)?;

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

            self.emit_instruction(previous_instruction, previous_position);
            self.emit_instruction(
                Instruction::set_local(self.current_register, local_index),
                start_position,
            );
            self.increment_register()?;

            self.parsed_expression = false;
        } else {
            self.emit_instruction(
                Instruction::get_local(self.current_register, local_index),
                self.previous_position,
            );

            self.parsed_expression = true;
        }

        Ok(())
    }

    fn parse_identifier_from(&mut self, token: Token, position: Span) -> Result<u8, ParseError> {
        if let Token::Identifier(text) = token {
            let identifier = Identifier::new(text);

            if let Ok(local_index) = self.chunk.get_local_index(&identifier, position) {
                Ok(local_index)
            } else {
                Err(ParseError::UndeclaredVariable {
                    identifier,
                    position,
                })
            }
        } else {
            Err(ParseError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: self.current_token.to_owned(),
                position,
            })
        }
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
            self.parse_statement(allowed, Context::None)?;
        }

        self.chunk.end_scope();

        Ok(())
    }

    fn parse_list(&mut self, _: Allowed) -> Result<(), ParseError> {
        let start = self.current_position.0;

        self.advance()?;

        let start_register = self.current_register;

        while !self.allow(Token::RightSquareBrace)? && !self.is_eof() {
            let next_register = self.current_register;

            self.parse_expression()?;

            if let Operation::LoadConstant = self.chunk.get_last_operation(self.current_position)? {
                self.increment_register()?;
            }

            if next_register != self.current_register.saturating_sub(1) {
                self.emit_instruction(
                    Instruction::close(next_register, self.current_register.saturating_sub(1)),
                    self.current_position,
                );
            }

            self.allow(Token::Comma)?;
        }

        let end_register = self.current_register.saturating_sub(1);
        let end = self.current_position.1;

        self.emit_instruction(
            Instruction::load_list(self.current_register, start_register, end_register),
            Span(start, end),
        );
        self.increment_register()?;

        self.parsed_expression = true;

        Ok(())
    }

    fn parse_if(&mut self, allowed: Allowed) -> Result<(), ParseError> {
        self.advance()?;
        self.parse_expression()?;

        let length = self.chunk.len();
        let is_explicit_boolean =
            matches!(self.previous_token, Token::Boolean(_)) && length == self.chunk.len() - 1;

        if is_explicit_boolean {
            self.emit_instruction(
                Instruction::test(self.current_register, false),
                self.current_position,
            );
        }

        let block_allowed = Allowed {
            assignment: allowed.assignment,
            explicit_return: allowed.explicit_return,
            implicit_return: false,
        };

        if let Token::LeftCurlyBrace = self.current_token {
            self.parse_block(block_allowed)?;
        }

        let last_operation = self.chunk.get_last_operation(self.current_position)?;

        if let (Operation::LoadConstant | Operation::LoadBoolean, Token::Else) =
            (last_operation, self.current_token)
        {
            let (mut load_constant, load_constant_position) =
                self.chunk.pop_instruction(self.current_position)?;

            load_constant.set_c_to_boolean(true);

            self.emit_instruction(load_constant, load_constant_position);
        }

        if self.allow(Token::Else)? {
            if let Token::If = self.current_token {
                self.parse_if(allowed)?;
            }

            if let Token::LeftCurlyBrace = self.current_token {
                self.parse_block(block_allowed)?;

                self.parsed_expression = true;
            }
        } else {
            self.parsed_expression = false;
        }

        Ok(())
    }

    fn parse_while(&mut self, allowed: Allowed) -> Result<(), ParseError> {
        self.advance()?;

        let jump_start = self.chunk.len();

        self.parse_expression()?;
        self.parse_block(Allowed {
            assignment: true,
            explicit_return: allowed.explicit_return,
            implicit_return: false,
        })?;

        let jump_end = self.chunk.len();
        let jump_distance = jump_end.abs_diff(jump_start) as u8;
        let jump_back = Instruction::jump(jump_distance, false);
        let jump_over_index = self.chunk.find_last_instruction(Operation::Jump);

        if let Some(index) = jump_over_index {
            let (_, jump_over_position) = self.chunk.remove_instruction(index);
            let jump_over = Instruction::jump(jump_distance - 1, true);

            self.chunk
                .insert_instruction(index, jump_over, jump_over_position);
        }

        self.chunk
            .insert_instruction(jump_end, jump_back, self.current_position);

        self.parsed_expression = false;

        Ok(())
    }

    fn parse_statement(&mut self, allowed: Allowed, context: Context) -> Result<(), ParseError> {
        self.parse(
            Precedence::None,
            Allowed {
                assignment: true,
                explicit_return: true,
                implicit_return: true,
            },
        )?;

        let previous_instructions = self.chunk.get_last_n_instructions();

        if let [Some((jump, _)), Some((comparison, comparison_position))] = previous_instructions {
            if let (Operation::Jump, Operation::Equal | Operation::Less | Operation::LessEqual) =
                (jump.operation(), comparison.operation())
            {
                if matches!(self.current_token, Token::Eof | Token::RightCurlyBrace)
                    || context == Context::Assignment
                {
                    let comparison_position = *comparison_position;

                    self.emit_instruction(
                        Instruction::load_boolean(self.current_register, true, true),
                        comparison_position,
                    );
                    self.emit_instruction(
                        Instruction::load_boolean(self.current_register, false, false),
                        comparison_position,
                    );
                }
            }
        }

        let end_of_statement = matches!(self.current_token, Token::Eof | Token::RightCurlyBrace);
        let has_semicolon = self.allow(Token::Semicolon)?;
        let returned = self.chunk.get_last_operation(self.current_position)? == Operation::Return;

        if allowed.implicit_return
            && self.parsed_expression
            && end_of_statement
            && !has_semicolon
            && !returned
        {
            self.emit_instruction(Instruction::r#return(true), self.current_position);
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

        self.parsed_expression = false;

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
        let register = self.current_register;

        self.expect(Token::Equal)?;

        self.parse_statement(
            Allowed {
                assignment: false,
                explicit_return: true,
                implicit_return: false,
            },
            Context::Assignment,
        )?;

        if self.current_register == register {
            self.increment_register()?;
        }

        let local_index = self
            .chunk
            .declare_local(identifier, r#type, is_mutable, register, position)?;

        self.emit_instruction(
            Instruction::define_local(self.current_register - 1, local_index, is_mutable),
            position,
        );

        self.parsed_expression = false;

        Ok(())
    }

    fn parse_function(&mut self, _: Allowed) -> Result<(), ParseError> {
        let function_start = self.current_position.0;
        let mut function_parser = Parser::new(self.lexer)?;

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

            function_parser.chunk.declare_local(
                parameter,
                Some(r#type),
                is_mutable,
                function_parser.current_register,
                Span(start, end),
            )?;

            function_parser.increment_register()?;
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
            function_parser.parse_statement(
                Allowed {
                    assignment: true,
                    explicit_return: true,
                    implicit_return: true,
                },
                Context::None,
            )?;
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
        let function = Value::function(function_parser.chunk, function_type);
        let function_end = self.current_position.1;

        self.lexer.skip_to(function_end);
        self.emit_constant(function, Span(function_start, function_end))?;

        self.parsed_expression = true;

        Ok(())
    }

    fn parse_call(&mut self) -> Result<(), ParseError> {
        let last_instruction = self.chunk.get_last_instruction(self.current_position)?.0;

        if !last_instruction.is_expression() {
            return Err(ParseError::ExpectedExpression {
                found: self.previous_token.to_owned(),
                position: self.previous_position,
            });
        }

        let start = self.current_position.0;

        self.advance()?;

        let function_register = self.current_register;

        self.increment_register()?;

        while !self.allow(Token::RightParenthesis)? {
            self.parse_expression()?;
            self.allow(Token::Comma)?;
        }

        let end = self.current_position.1;

        self.emit_instruction(
            Instruction::call(self.current_register, function_register),
            Span(start, end),
        );
        self.increment_register()?;

        self.parsed_expression = true;

        Ok(())
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
                prefix: Some(Parser::expect_expression),
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
