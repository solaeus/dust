#[cfg(test)]
mod tests;

use std::{
    fmt::{self, Display, Formatter},
    mem::replace,
    num::{ParseFloatError, ParseIntError},
};

use colored::Colorize;

use crate::{
    AnnotatedError, Chunk, ChunkError, DustError, Identifier, Instruction, LexError, Lexer,
    Operation, Span, Token, TokenKind, TokenOwned, Value,
};

pub fn parse(source: &str) -> Result<Chunk, DustError> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer).map_err(|error| DustError::Parse { error, source })?;

    while !parser.is_eof() {
        parser
            .parse_statement(true)
            .map_err(|error| DustError::Parse { error, source })?;
    }

    Ok(parser.chunk)
}

#[derive(Debug)]
pub struct Parser<'src> {
    chunk: Chunk,
    lexer: Lexer<'src>,
    current_register: u8,
    current_token: Token<'src>,
    current_position: Span,
    previous_token: Token<'src>,
    previous_position: Span,
}

impl<'src> Parser<'src> {
    pub fn new(mut lexer: Lexer<'src>) -> Result<Self, ParseError> {
        let (current_token, current_position) = lexer.next_token()?;

        log::info!(
            "{} at {}",
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
        })
    }

    pub fn take_chunk(self) -> Chunk {
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

    fn decrement_register(&mut self) -> Result<(), ParseError> {
        let current = self.current_register;

        if current == 0 {
            Err(ParseError::RegisterUnderflow {
                position: self.current_position,
            })
        } else {
            self.current_register -= 1;

            Ok(())
        }
    }

    fn advance(&mut self) -> Result<(), ParseError> {
        if self.is_eof() {
            return Ok(());
        }

        let (new_token, position) = self.lexer.next_token()?;

        log::info!(
            "{} at {}",
            new_token.to_string().bold(),
            position.to_string()
        );

        self.previous_token = replace(&mut self.current_token, new_token);
        self.previous_position = replace(&mut self.current_position, position);

        Ok(())
    }

    fn allow(&mut self, allowed: TokenKind) -> Result<bool, ParseError> {
        if self.current_token.kind() == allowed {
            self.advance()?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn expect(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        if self.current_token.kind() == expected {
            self.advance()
        } else {
            Err(ParseError::ExpectedToken {
                expected,
                found: self.current_token.to_owned(),
                position: self.current_position,
            })
        }
    }

    fn emit_instruction(&mut self, instruction: Instruction, position: Span) {
        self.chunk.push_instruction(instruction, position);
    }

    fn emit_constant(&mut self, value: Value) -> Result<(), ParseError> {
        let position = self.previous_position;
        let constant_index = self.chunk.push_constant(value, position)?;

        self.emit_instruction(
            Instruction::load_constant(self.current_register, constant_index, false),
            position,
        );

        Ok(())
    }

    fn parse_boolean(
        &mut self,
        _allow_assignment: bool,
        _allow_return: bool,
    ) -> Result<(), ParseError> {
        let boolean_text = if let Token::Boolean(text) = self.current_token {
            text
        } else {
            return Err(ParseError::ExpectedToken {
                expected: TokenKind::Boolean,
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        };

        let position = self.current_position;
        let boolean = boolean_text.parse::<bool>().unwrap();

        self.advance()?;

        self.emit_instruction(
            Instruction::load_boolean(self.current_register, boolean, false),
            position,
        );

        Ok(())
    }

    fn parse_byte(
        &mut self,
        _allow_assignment: bool,
        _allow_return: bool,
    ) -> Result<(), ParseError> {
        if let Token::Byte(text) = self.current_token {
            self.advance()?;

            let byte =
                u8::from_str_radix(&text[2..], 16).map_err(|error| ParseError::ParseIntError {
                    error,
                    position: self.previous_position,
                })?;
            let value = Value::byte(byte);

            self.emit_constant(value)?;
        }

        Ok(())
    }

    fn parse_character(
        &mut self,
        _allow_assignment: bool,
        _allow_return: bool,
    ) -> Result<(), ParseError> {
        if let Token::Character(character) = self.current_token {
            self.advance()?;

            let value = Value::character(character);

            self.emit_constant(value)?;
        }

        Ok(())
    }

    fn parse_float(
        &mut self,
        _allow_assignment: bool,
        _allow_return: bool,
    ) -> Result<(), ParseError> {
        if let Token::Float(text) = self.current_token {
            self.advance()?;

            let float = text
                .parse::<f64>()
                .map_err(|error| ParseError::ParseFloatError {
                    error,
                    position: self.previous_position,
                })?;
            let value = Value::float(float);

            self.emit_constant(value)?;
        }

        Ok(())
    }

    fn parse_integer(
        &mut self,
        _allow_assignment: bool,
        _allow_return: bool,
    ) -> Result<(), ParseError> {
        if let Token::Integer(text) = self.current_token {
            self.advance()?;

            let integer = text
                .parse::<i64>()
                .map_err(|error| ParseError::ParseIntError {
                    error,
                    position: self.previous_position,
                })?;
            let value = Value::integer(integer);

            self.emit_constant(value)?;
        }

        Ok(())
    }

    fn parse_string(
        &mut self,
        _allow_assignment: bool,
        _allow_return: bool,
    ) -> Result<(), ParseError> {
        if let Token::String(text) = self.current_token {
            self.advance()?;

            let value = Value::string(text);

            self.emit_constant(value)?;
        }

        Ok(())
    }

    fn parse_grouped(
        &mut self,
        _allow_assignment: bool,
        _allow_return: bool,
    ) -> Result<(), ParseError> {
        self.allow(TokenKind::LeftParenthesis)?;
        self.parse_expression()?;
        self.expect(TokenKind::RightParenthesis)
    }

    fn parse_unary(
        &mut self,
        _allow_assignment: bool,
        _allow_return: bool,
    ) -> Result<(), ParseError> {
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

        Ok(())
    }

    fn handle_binary_argument(
        &mut self,
        instruction: &Instruction,
    ) -> Result<(bool, bool, u8), ParseError> {
        let mut push_back = false;
        let mut is_constant = false;
        let argument = match instruction.operation() {
            Operation::GetLocal => {
                let local_index = instruction.b();
                let local = self.chunk.get_local(local_index, self.current_position)?;

                if let Some(index) = local.register_index {
                    index
                } else {
                    instruction.a()
                }
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

        Ok((push_back, is_constant, argument))
    }

    fn parse_math_binary(&mut self) -> Result<(), ParseError> {
        let (left_instruction, left_position) =
            self.chunk.pop_instruction(self.current_position)?;
        let (push_back_left, left_is_constant, left) =
            self.handle_binary_argument(&left_instruction)?;

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(&operator.kind());

        self.advance()?;
        self.parse(rule.precedence.increment())?;

        let mut new_instruction = match operator.kind() {
            TokenKind::Plus => Instruction::add(self.current_register, left, 0),
            TokenKind::Minus => Instruction::subtract(self.current_register, left, 0),
            TokenKind::Star => Instruction::multiply(self.current_register, left, 0),
            TokenKind::Slash => Instruction::divide(self.current_register, left, 0),
            TokenKind::Percent => Instruction::modulo(self.current_register, left, 0),
            _ => {
                return Err(ParseError::ExpectedTokenMultiple {
                    expected: &[
                        TokenKind::Plus,
                        TokenKind::Minus,
                        TokenKind::Star,
                        TokenKind::Slash,
                        TokenKind::Percent,
                    ],
                    found: operator.to_owned(),
                    position: operator_position,
                })
            }
        };

        self.increment_register()?;

        let (right_instruction, right_position) =
            self.chunk.pop_instruction(self.current_position)?;
        let (push_back_right, right_is_constant, right) =
            self.handle_binary_argument(&right_instruction)?;

        new_instruction.set_c(right);

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

        while let Ok(operation) = self.chunk.get_last_operation() {
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
        let is_repetition = matches!(
            self.chunk.get_last_n_operations(),
            [
                Some(_),
                Some(_),
                Some(Operation::Jump),
                Some(Operation::Equal | Operation::Less | Operation::LessEqual)
            ]
        );

        if is_repetition {
            self.decrement_register()?;
            self.decrement_register()?;
        }

        let (left_instruction, left_position) =
            self.chunk.pop_instruction(self.current_position)?;
        let (push_back_left, left_is_constant, left) =
            self.handle_binary_argument(&left_instruction)?;

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(&operator.kind());
        let mut instruction = match self.current_token.kind() {
            TokenKind::DoubleEqual => Instruction::equal(true, left.saturating_sub(1), 0),
            TokenKind::BangEqual => Instruction::equal(false, left.saturating_sub(1), 0),
            TokenKind::Less => Instruction::less(true, left.saturating_sub(1), 0),
            TokenKind::LessEqual => Instruction::less_equal(true, left.saturating_sub(1), 0),
            TokenKind::Greater => Instruction::less_equal(false, left.saturating_sub(1), 0),
            TokenKind::GreaterEqual => Instruction::less(false, left.saturating_sub(1), 0),
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
                    found: self.current_token.to_owned(),
                    position: self.current_position,
                })
            }
        };

        self.advance()?;
        self.parse(rule.precedence.increment())?;

        let (right_instruction, right_position) =
            self.chunk.pop_instruction(self.current_position)?;
        let (push_back_right, right_is_constant, right) =
            self.handle_binary_argument(&right_instruction)?;

        instruction.set_c(right);

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
        self.emit_instruction(
            Instruction::load_boolean(self.current_register, true, true),
            operator_position,
        );
        self.emit_instruction(
            Instruction::load_boolean(self.current_register, false, false),
            operator_position,
        );
        self.increment_register()?;

        Ok(())
    }

    fn parse_logical_binary(&mut self) -> Result<(), ParseError> {
        let (left_instruction, left_position) =
            self.chunk.pop_instruction(self.current_position)?;

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(&operator.kind());

        let instruction = match operator.kind() {
            TokenKind::DoubleAmpersand => Instruction::test(left_instruction.a(), false),
            TokenKind::DoublePipe => Instruction::test(left_instruction.a(), true),
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
        self.parse(rule.precedence.increment())?;

        let (right_instruction, right_position) =
            self.chunk.pop_instruction(self.current_position)?;

        self.emit_instruction(left_instruction, left_position);
        self.emit_instruction(instruction, operator_position);
        self.emit_instruction(Instruction::jump(1, true), operator_position);
        self.emit_instruction(right_instruction, right_position);

        Ok(())
    }

    fn parse_variable(
        &mut self,
        allow_assignment: bool,
        _allow_return: bool,
    ) -> Result<(), ParseError> {
        let token = self.current_token;
        let start_position = self.current_position;

        self.advance()?;

        let local_index = self.parse_identifier_from(token, start_position)?;

        if allow_assignment && self.allow(TokenKind::Equal)? {
            let is_mutable = self.chunk.get_local(local_index, start_position)?.mutable;

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
                let previous_register = self
                    .chunk
                    .get_local(local_index, start_position)?
                    .register_index;

                if let Some(register_index) = previous_register {
                    log::trace!("Condensing SET_LOCAL to binary expression");

                    previous_instruction.set_a(register_index);
                    self.emit_instruction(previous_instruction, self.current_position);

                    return Ok(());
                }
            }

            self.emit_instruction(previous_instruction, previous_position);
            self.emit_instruction(
                Instruction::set_local(self.current_register, local_index),
                start_position,
            );
        } else {
            self.emit_instruction(
                Instruction::get_local(self.current_register, local_index),
                self.previous_position,
            );
        }

        Ok(())
    }

    fn parse_identifier_from(&mut self, token: Token, position: Span) -> Result<u8, ParseError> {
        if let Token::Identifier(text) = token {
            let identifier = Identifier::new(text);

            if let Ok(local_index) = self.chunk.get_local_index(&identifier, position) {
                Ok(local_index)
            } else {
                Err(ParseError::UndefinedVariable {
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

    fn parse_block(
        &mut self,
        _allow_assignment: bool,
        _allow_return: bool,
    ) -> Result<(), ParseError> {
        self.advance()?;
        self.chunk.begin_scope();

        while !self.allow(TokenKind::RightCurlyBrace)? && !self.is_eof() {
            self.parse_statement(_allow_return)?;
        }

        self.chunk.end_scope();

        Ok(())
    }

    fn parse_list(
        &mut self,
        _allow_assignment: bool,
        _allow_return: bool,
    ) -> Result<(), ParseError> {
        let start = self.current_position.0;

        self.advance()?;

        let start_register = self.current_register;

        while !self.allow(TokenKind::RightSquareBrace)? && !self.is_eof() {
            let next_register = self.current_register;

            self.parse(Precedence::Assignment)?; // Do not allow assignment

            if let Operation::LoadConstant = self.chunk.get_last_operation()? {
                self.increment_register()?;
            }

            if next_register != self.current_register.saturating_sub(1) {
                self.emit_instruction(
                    Instruction::close(next_register, self.current_register.saturating_sub(1)),
                    self.current_position,
                );
            }

            self.allow(TokenKind::Comma)?;
        }

        let end_register = self.current_register - 1;
        let end = self.current_position.1;

        self.emit_instruction(
            Instruction::load_list(self.current_register, start_register, end_register),
            Span(start, end),
        );

        Ok(())
    }

    fn parse_if(&mut self, allow_assignment: bool, allow_return: bool) -> Result<(), ParseError> {
        let length = self.chunk.len();

        self.advance()?;
        self.parse_expression()?;

        let is_explicit_boolean =
            matches!(self.previous_token, Token::Boolean(_)) && length == self.chunk.len() - 1;

        if is_explicit_boolean {
            self.emit_instruction(
                Instruction::test(self.current_register, false),
                self.current_position,
            );
        }

        let jump_position = if matches!(
            self.chunk.get_last_n_operations(),
            [
                Some(Operation::LoadBoolean),
                Some(Operation::LoadBoolean),
                Some(Operation::Jump)
            ]
        ) {
            self.chunk.pop_instruction(self.current_position)?;
            self.chunk.pop_instruction(self.current_position)?;
            self.decrement_register()?;
            self.chunk.pop_instruction(self.current_position)?.1
        } else {
            self.current_position
        };

        let jump_start = self.chunk.len();

        if let Token::LeftCurlyBrace = self.current_token {
            self.parse_block(allow_assignment, allow_return)?;
        }

        if self.chunk.get_last_operation()? == Operation::LoadConstant
            && self.current_token == Token::Else
        {
            let (mut load_constant, load_constant_position) =
                self.chunk.pop_instruction(self.current_position)?;

            load_constant.set_c_to_boolean(true);

            self.emit_instruction(load_constant, load_constant_position);
        }

        let jump_end = self.chunk.len();
        let jump_distance = jump_end.saturating_sub(jump_start);

        self.chunk.insert_instruction(
            jump_start,
            Instruction::jump(jump_distance as u8, true),
            jump_position,
        );

        if self.allow(TokenKind::Else)? {
            if let Token::If = self.current_token {
                self.parse_if(allow_assignment, allow_return)?;
            }

            if let Token::LeftCurlyBrace = self.current_token {
                self.parse_block(allow_assignment, allow_return)?;
            }
        }

        Ok(())
    }

    fn parse_while(
        &mut self,
        allow_assignment: bool,
        allow_return: bool,
    ) -> Result<(), ParseError> {
        self.advance()?;

        let jump_start = self.chunk.len();

        self.parse_expression()?;
        self.parse_block(allow_assignment, allow_return)?;

        let jump_end = self.chunk.len() - 1;
        let jump_distance = jump_end.saturating_sub(jump_start) as u8;
        let jump_back = Instruction::jump(jump_distance, false);
        let jump_over_index = self.chunk.find_last_instruction(Operation::Jump);

        if let Some(index) = jump_over_index {
            let (mut jump_over, jump_over_position) = self.chunk.remove_instruction(index);

            jump_over.set_b(jump_distance);
            self.chunk
                .insert_instruction(index, jump_over, jump_over_position);
        }

        self.chunk
            .insert_instruction(jump_end, jump_back, self.current_position);

        Ok(())
    }

    fn parse_expression(&mut self) -> Result<(), ParseError> {
        self.parse(Precedence::None)
    }

    fn parse_statement(&mut self, allow_return: bool) -> Result<(), ParseError> {
        match self.current_token {
            Token::Let => {
                self.parse_let_statement(true, allow_return)?;
            }
            Token::LeftCurlyBrace => {
                self.parse_block(true, true)?;
            }
            _ => {
                self.parse_expression()?;
            }
        };

        self.allow(TokenKind::Semicolon)?;

        Ok(())
    }

    fn parse_let_statement(
        &mut self,
        allow_assignment: bool,
        _allow_return: bool,
    ) -> Result<(), ParseError> {
        if !allow_assignment {
            return Err(ParseError::ExpectedExpression {
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        }

        self.allow(TokenKind::Let)?;

        let is_mutable = self.allow(TokenKind::Mut)?;
        let position = self.current_position;
        let identifier = if let Token::Identifier(text) = self.current_token {
            self.advance()?;

            Identifier::new(text)
        } else {
            return Err(ParseError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        };

        self.expect(TokenKind::Equal)?;
        self.parse_expression()?;
        self.increment_register()?;

        let (previous_instruction, previous_position) = self.chunk.get_last_instruction()?;
        let register = previous_instruction.a();
        let local_index =
            self.chunk
                .declare_local(identifier, is_mutable, register, *previous_position)?;

        // Optimize for assignment to a comparison
        // if let Operation::Jump = previous_instruction.operation() {
        //     let (jump, jump_position) = self.chunk.pop_instruction(self.current_position)?;

        //     if let Some(Operation::Equal) = self.chunk.get_last_operation() {
        //         self.emit_instruction(jump, jump_position);
        //         self.emit_instruction(
        //             Instruction::load_boolean(self.current_register, true, true),
        //             self.current_position,
        //         );
        //         self.emit_instruction(
        //             Instruction::load_boolean(self.current_register, false, false),
        //             self.current_position,
        //         );
        //     } else {
        //         self.emit_instruction(jump, jump_position);
        //     }
        // }

        self.emit_instruction(
            Instruction::define_local(register, local_index, is_mutable),
            position,
        );
        self.allow(TokenKind::Semicolon)?;

        Ok(())
    }

    fn parse(&mut self, precedence: Precedence) -> Result<(), ParseError> {
        let allow_assignment = precedence < Precedence::Assignment;
        let allow_return = precedence == Precedence::None;

        if let Some(prefix_parser) = ParseRule::from(&self.current_token.kind()).prefix {
            log::debug!(
                "{} is {precedence} prefix",
                self.current_token.to_string().bold(),
            );

            prefix_parser(self, allow_assignment, allow_return)?;
        }

        let mut infix_rule = ParseRule::from(&self.current_token.kind());

        while precedence <= infix_rule.precedence {
            if let Some(infix_parser) = infix_rule.infix {
                log::debug!(
                    "{} is {precedence} infix",
                    self.current_token.to_string().bold(),
                );

                if allow_assignment && self.current_token == Token::Equal {
                    return Err(ParseError::InvalidAssignmentTarget {
                        found: self.current_token.to_owned(),
                        position: self.current_position,
                    });
                }

                infix_parser(self)?;
            } else {
                break;
            }

            infix_rule = ParseRule::from(&self.current_token.kind());
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

type PrefixFunction<'a> = fn(&mut Parser<'a>, bool, bool) -> Result<(), ParseError>;
type InfixFunction<'a> = fn(&mut Parser<'a>) -> Result<(), ParseError>;

#[derive(Debug, Clone, Copy)]
pub struct ParseRule<'a> {
    pub prefix: Option<PrefixFunction<'a>>,
    pub infix: Option<InfixFunction<'a>>,
    pub precedence: Precedence,
}

impl From<&TokenKind> for ParseRule<'_> {
    fn from(token_kind: &TokenKind) -> Self {
        match token_kind {
            TokenKind::Async => todo!(),
            TokenKind::Bang => ParseRule {
                prefix: Some(Parser::parse_unary),
                infix: None,
                precedence: Precedence::Unary,
            },
            TokenKind::BangEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Equality,
            },
            TokenKind::Bool => todo!(),
            TokenKind::Boolean => ParseRule {
                prefix: Some(Parser::parse_boolean),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Break => todo!(),
            TokenKind::Byte => ParseRule {
                prefix: Some(Parser::parse_byte),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Character => ParseRule {
                prefix: Some(Parser::parse_character),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Colon => todo!(),
            TokenKind::Comma => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Dot => todo!(),
            TokenKind::DoubleAmpersand => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_logical_binary),
                precedence: Precedence::LogicalAnd,
            },
            TokenKind::DoubleEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Equality,
            },
            TokenKind::DoublePipe => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_logical_binary),
                precedence: Precedence::LogicalOr,
            },
            TokenKind::DoubleDot => todo!(),
            TokenKind::Eof => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Equal => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::Assignment,
            },
            TokenKind::Else => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Float => ParseRule {
                prefix: Some(Parser::parse_float),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::FloatKeyword => todo!(),
            TokenKind::Greater => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            TokenKind::GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            TokenKind::Identifier => ParseRule {
                prefix: Some(Parser::parse_variable),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::If => ParseRule {
                prefix: Some(Parser::parse_if),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Int => todo!(),
            TokenKind::Integer => ParseRule {
                prefix: Some(Parser::parse_integer),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::LeftCurlyBrace => ParseRule {
                prefix: Some(Parser::parse_block),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::LeftParenthesis => ParseRule {
                prefix: Some(Parser::parse_grouped),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::LeftSquareBrace => ParseRule {
                prefix: Some(Parser::parse_list),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Less => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            TokenKind::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            TokenKind::Let => ParseRule {
                prefix: Some(Parser::parse_let_statement),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Loop => todo!(),
            TokenKind::Map => todo!(),
            TokenKind::Minus => ParseRule {
                prefix: Some(Parser::parse_unary),
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Term,
            },
            TokenKind::MinusEqual => todo!(),
            TokenKind::Mut => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Percent => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Factor,
            },
            TokenKind::Plus => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Term,
            },
            TokenKind::PlusEqual => todo!(),
            TokenKind::RightCurlyBrace => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::RightParenthesis => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::RightSquareBrace => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Semicolon => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Slash => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Factor,
            },
            TokenKind::Star => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Factor,
            },
            TokenKind::Str => todo!(),
            TokenKind::String => ParseRule {
                prefix: Some(Parser::parse_string),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Struct => todo!(),
            TokenKind::While => ParseRule {
                prefix: Some(Parser::parse_while),
                infix: None,
                precedence: Precedence::None,
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
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
    InvalidAssignmentTarget {
        found: TokenOwned,
        position: Span,
    },
    UndefinedVariable {
        identifier: Identifier,
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
            Self::CannotMutateImmutableVariable { .. } => "Cannot mutate immutable variable",
            Self::ExpectedExpression { .. } => "Expected an expression",
            Self::ExpectedToken { .. } => "Expected a specific token",
            Self::ExpectedTokenMultiple { .. } => "Expected one of multiple tokens",
            Self::InvalidAssignmentTarget { .. } => "Invalid assignment target",
            Self::UndefinedVariable { .. } => "Undefined variable",
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
            Self::CannotMutateImmutableVariable { identifier, .. } => {
                Some(format!("Cannot mutate immutable variable \"{identifier}\""))
            }
            Self::ExpectedExpression { found, .. } => Some(format!("Found \"{found}\"")),
            Self::ExpectedToken {
                expected, found, ..
            } => Some(format!("Expected \"{expected}\", found \"{found}\"")),
            Self::ExpectedTokenMultiple {
                expected, found, ..
            } => {
                let mut details = String::from("Expected");

                for (index, token) in expected.iter().enumerate() {
                    details.push_str(&format!(" \"{token}\""));

                    if index < expected.len() - 2 {
                        details.push_str(", ");
                    }

                    if index == expected.len() - 2 {
                        details.push_str(" or");
                    }
                }

                details.push_str(&format!(" found \"{found}\""));

                Some(details)
            }
            Self::InvalidAssignmentTarget { found, .. } => {
                Some(format!("Invalid assignment target, found \"{found}\""))
            }
            Self::UndefinedVariable { identifier, .. } => {
                Some(format!("Undefined variable \"{identifier}\""))
            }
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
            Self::CannotMutateImmutableVariable { position, .. } => *position,
            Self::ExpectedExpression { position, .. } => *position,
            Self::ExpectedToken { position, .. } => *position,
            Self::ExpectedTokenMultiple { position, .. } => *position,
            Self::InvalidAssignmentTarget { position, .. } => *position,
            Self::UndefinedVariable { position, .. } => *position,
            Self::RegisterOverflow { position } => *position,
            Self::RegisterUnderflow { position } => *position,
            Self::Chunk(error) => error.position(),
            Self::Lex(error) => error.position(),
            Self::ParseFloatError { position, .. } => *position,
            Self::ParseIntError { position, .. } => *position,
        }
    }
}

impl From<LexError> for ParseError {
    fn from(error: LexError) -> Self {
        Self::Lex(error)
    }
}
