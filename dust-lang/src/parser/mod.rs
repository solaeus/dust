#[cfg(test)]
mod tests;

use std::{
    fmt::{self, Display, Formatter},
    mem::replace,
    num::{ParseFloatError, ParseIntError},
};

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

        log::trace!("Starting parser with token \"{current_token}\" at {current_position}");

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

        log::trace!("Parsing \"{new_token}\" at {position}");

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
            Instruction::load_constant(self.current_register, constant_index),
            position,
        );
        self.increment_register()?;

        Ok(())
    }

    fn parse_boolean(
        &mut self,
        _allow_assignment: bool,
        _allow_return: bool,
    ) -> Result<(), ParseError> {
        if let Token::Boolean(text) = self.current_token {
            let position = self.current_position;
            let boolean = text.parse::<bool>().unwrap();

            self.advance()?;

            let previous_operations = self.chunk.get_last_n_operations::<2>();

            if let [Some(Operation::LoadBoolean), Some(Operation::LoadBoolean)] =
                previous_operations
            {
                let (second_boolean, second_position) =
                    self.chunk.pop_instruction(self.current_position)?;
                let (first_boolean, first_position) =
                    self.chunk.pop_instruction(self.current_position)?;

                if first_boolean.first_argument_as_boolean() == boolean {
                    let skip = first_boolean.second_argument_as_boolean();

                    self.emit_instruction(
                        Instruction::load_boolean(self.current_register, boolean, skip),
                        position,
                    );

                    return Ok(());
                }

                if second_boolean.first_argument_as_boolean() == boolean {
                    let skip = second_boolean.second_argument_as_boolean();

                    self.emit_instruction(
                        Instruction::load_boolean(self.current_register, boolean, skip),
                        position,
                    );

                    return Ok(());
                }

                self.emit_instruction(first_boolean, first_position);
                self.emit_instruction(second_boolean, second_position);
            }

            let skip = previous_operations[0] == Some(Operation::Jump);

            self.emit_instruction(
                Instruction::load_boolean(self.current_register, boolean, skip),
                position,
            );
        }

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

        let (is_constant, destination, from_register) = match previous_instruction.operation() {
            Operation::LoadConstant => {
                self.decrement_register()?;

                (
                    true,
                    previous_instruction.destination(),
                    previous_instruction.first_argument(),
                )
            }
            _ => {
                self.emit_instruction(previous_instruction, previous_position);

                (false, self.current_register, self.current_register - 1)
            }
        };

        let mut instruction = match operator.kind() {
            TokenKind::Bang => Instruction::not(destination, from_register),
            TokenKind::Minus => Instruction::negate(destination, from_register),
            _ => {
                return Err(ParseError::ExpectedTokenMultiple {
                    expected: &[TokenKind::Bang, TokenKind::Minus],
                    found: operator.to_owned(),
                    position: operator_position,
                })
            }
        };

        if is_constant {
            instruction.set_first_argument_to_constant();
        }

        self.increment_register()?;
        self.emit_instruction(instruction, operator_position);

        Ok(())
    }

    fn parse_binary(&mut self) -> Result<(), ParseError> {
        fn handle_argument(
            parser: &mut Parser,
            instruction: &Instruction,
        ) -> Result<(bool, bool, u8), ParseError> {
            let mut push_back = false;
            let mut is_constant = false;
            let argument = match instruction.operation() {
                Operation::GetLocal => {
                    parser.decrement_register()?;
                    instruction.destination()
                }
                Operation::LoadConstant => {
                    is_constant = true;

                    parser.decrement_register()?;
                    instruction.first_argument()
                }
                Operation::LoadBoolean => {
                    is_constant = true;
                    push_back = true;

                    instruction.destination()
                }
                Operation::Close => {
                    return Err(ParseError::ExpectedExpression {
                        found: parser.previous_token.to_owned(),
                        position: parser.previous_position,
                    });
                }
                _ => {
                    push_back = true;

                    instruction.destination()
                }
            };

            Ok((push_back, is_constant, argument))
        }

        let (left_instruction, left_position) =
            self.chunk.pop_instruction(self.current_position)?;
        let (push_back_left, left_is_constant, left) = handle_argument(self, &left_instruction)?;

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(&operator.kind());

        let (mut instruction, is_comparison) = match operator.kind() {
            TokenKind::Plus => (Instruction::add(self.current_register, left, 0), false),
            TokenKind::Minus => (Instruction::subtract(self.current_register, left, 0), false),
            TokenKind::Star => (Instruction::multiply(self.current_register, left, 0), false),
            TokenKind::Slash => (Instruction::divide(self.current_register, left, 0), false),
            TokenKind::Percent => (Instruction::modulo(self.current_register, left, 0), false),
            TokenKind::DoubleEqual => (Instruction::equal(true, left, 0), true),
            TokenKind::BangEqual => (Instruction::equal(false, left, 0), true),
            TokenKind::Less => (Instruction::less(true, left, 0), true),
            TokenKind::LessEqual => (Instruction::less_equal(true, left, 0), true),
            TokenKind::Greater => (Instruction::less_equal(false, left, 0), true),
            TokenKind::GreaterEqual => (Instruction::less(false, left, 0), true),
            TokenKind::DoubleAmpersand => {
                let and_test = Instruction::test(self.current_register, false);

                (and_test, true)
            }
            TokenKind::DoublePipe => {
                let or_test = Instruction::test(self.current_register, true);

                (or_test, true)
            }
            _ => {
                return Err(ParseError::ExpectedTokenMultiple {
                    expected: &[
                        TokenKind::Plus,
                        TokenKind::Minus,
                        TokenKind::Star,
                        TokenKind::Slash,
                        TokenKind::Percent,
                        TokenKind::DoubleEqual,
                        TokenKind::BangEqual,
                        TokenKind::Less,
                        TokenKind::LessEqual,
                        TokenKind::Greater,
                        TokenKind::GreaterEqual,
                        TokenKind::DoubleAmpersand,
                        TokenKind::DoublePipe,
                    ],
                    found: operator.to_owned(),
                    position: operator_position,
                })
            }
        };

        if !(operator == Token::DoubleEqual) {
            self.increment_register()?;
        }

        self.advance()?;
        self.parse(rule.precedence.increment())?;

        let (right_instruction, right_position) =
            self.chunk.pop_instruction(self.current_position)?;
        let (push_back_right, right_is_constant, right) =
            handle_argument(self, &right_instruction)?;

        instruction.set_second_argument(right);

        if left_is_constant {
            instruction.set_first_argument_to_constant();
        }

        if right_is_constant {
            instruction.set_second_argument_to_constant();
        }

        if !is_comparison {
            if push_back_left {
                self.emit_instruction(left_instruction, left_position);
            }

            if push_back_right {
                self.emit_instruction(right_instruction, right_position);
            }

            self.emit_instruction(instruction, operator_position);
        }

        if is_comparison {
            let push_left_first = self.current_register.saturating_sub(1) == left;

            if push_back_left && push_left_first {
                self.emit_instruction(left_instruction, left_position);
            }

            let jump_distance = if left_is_constant { 1 } else { 2 };

            self.emit_instruction(instruction, operator_position);
            self.emit_instruction(Instruction::jump(jump_distance, true), operator_position);

            if push_back_left && !push_left_first {
                self.emit_instruction(left_instruction, left_position);
            }

            if push_back_right {
                self.emit_instruction(right_instruction, right_position);
            }

            if !push_back_left && !push_back_right {
                if self.current_register > 0 {
                    self.decrement_register()?;
                }

                self.emit_instruction(
                    Instruction::load_boolean(self.current_register, true, true),
                    operator_position,
                );
                self.emit_instruction(
                    Instruction::load_boolean(self.current_register, false, false),
                    operator_position,
                );
            }
        }

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

            if previous_instruction.operation().is_binary() {
                let previous_register = self
                    .chunk
                    .get_local(local_index, start_position)?
                    .register_index;

                if let Some(register_index) = previous_register {
                    log::trace!("Condensing SET_LOCAL to binary expression");

                    previous_instruction.set_destination(register_index);
                    self.emit_instruction(previous_instruction, self.current_position);

                    return Ok(());
                }
            }

            self.emit_instruction(previous_instruction, previous_position);
            self.emit_instruction(
                Instruction::set_local(self.current_register - 1, local_index),
                start_position,
            );
        } else {
            self.emit_instruction(
                Instruction::get_local(self.current_register, local_index),
                self.previous_position,
            );
            self.increment_register()?;
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
        allow_return: bool,
    ) -> Result<(), ParseError> {
        self.advance()?;
        self.chunk.begin_scope();

        while !self.allow(TokenKind::RightCurlyBrace)? && !self.is_eof() {
            self.parse_statement(allow_return)?;
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
        let mut length = 0;

        while !self.allow(TokenKind::RightSquareBrace)? && !self.is_eof() {
            let next_register = self.current_register;

            self.parse(Precedence::Assignment)?; // Do not allow assignment

            if next_register != self.current_register - 1 {
                self.emit_instruction(
                    Instruction::close(next_register, self.current_register - 1),
                    self.current_position,
                );
            }

            length += 1;

            if !self.allow(TokenKind::Comma)? {
                self.expect(TokenKind::RightSquareBrace)?;

                break;
            }
        }

        let end = self.current_position.1;

        self.emit_instruction(
            Instruction::load_list(self.current_register, start_register, length),
            Span(start, end),
        );
        self.increment_register()?;

        Ok(())
    }

    fn parse_if(&mut self, allow_assignment: bool, allow_return: bool) -> Result<(), ParseError> {
        self.advance()?;
        self.parse_expression()?;

        let (second_load_boolean, second_position) =
            self.chunk.pop_instruction(self.current_position)?;
        let (first_load_boolean, first_position) =
            self.chunk.pop_instruction(self.current_position)?;
        let length_after_expression = self.chunk.len();

        self.parse_block(allow_assignment, allow_return)?;

        let jump_start = self.current_register;
        let jump_index = self.chunk.len();

        if self.allow(TokenKind::Else)? {
            if self.allow(TokenKind::If)? {
                self.parse_if(allow_assignment, allow_return)?;
            } else {
                self.parse_block(allow_assignment, allow_return)?;
            }
        }

        if self.chunk.len() == length_after_expression {
            self.emit_instruction(first_load_boolean, first_position);
            self.emit_instruction(second_load_boolean, second_position);
        }

        if let Some(Operation::LoadBoolean) = self.chunk.get_last_operation() {
            // Skip the jump if the last instruction was a LoadBoolean operation. A LoadBoolean can
            // skip the following instruction, so a jump is unnecessary.
        } else {
            let jump_end = self.current_register;
            let jump_distance = (jump_end - jump_start).max(1);
            let jump = Instruction::jump(jump_distance, true);

            self.chunk
                .insert_instruction(jump_index, jump, self.current_position);
        }

        Ok(())
    }

    fn parse_while(
        &mut self,
        allow_assignment: bool,
        allow_return: bool,
    ) -> Result<(), ParseError> {
        self.advance()?;
        self.parse_expression()?;
        self.parse_block(allow_assignment, allow_return)?;

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

        let (previous_instruction, previous_position) =
            *self
                .chunk
                .get_previous()
                .ok_or_else(|| ParseError::ExpectedExpression {
                    found: self.current_token.to_owned(),
                    position,
                })?;
        let register = previous_instruction.destination();
        let local_index =
            self.chunk
                .declare_local(identifier, is_mutable, register, previous_position)?;

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
            log::trace!(
                "Parsing \"{}\" as prefix at precedence {precedence}",
                self.current_token,
            );

            prefix_parser(self, allow_assignment, allow_return)?;
        }

        let mut infix_rule = ParseRule::from(&self.current_token.kind());

        while precedence <= infix_rule.precedence {
            if let Some(infix_parser) = infix_rule.infix {
                log::trace!(
                    "Parsing \"{}\" as infix at precedence {precedence}",
                    self.current_token,
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
                infix: Some(Parser::parse_binary),
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
                infix: Some(Parser::parse_binary),
                precedence: Precedence::LogicalAnd,
            },
            TokenKind::DoubleEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary),
                precedence: Precedence::Equality,
            },
            TokenKind::DoublePipe => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary),
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
                infix: Some(Parser::parse_binary),
                precedence: Precedence::Comparison,
            },
            TokenKind::GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary),
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
                infix: Some(Parser::parse_binary),
                precedence: Precedence::Comparison,
            },
            TokenKind::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary),
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
                infix: Some(Parser::parse_binary),
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
                infix: Some(Parser::parse_binary),
                precedence: Precedence::Factor,
            },
            TokenKind::Plus => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary),
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
                infix: Some(Parser::parse_binary),
                precedence: Precedence::Factor,
            },
            TokenKind::Star => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary),
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
                let expected = expected
                    .iter()
                    .map(|kind| kind.to_string() + ", ")
                    .collect::<String>();

                Some(format!("Expected one of {expected}, found \"{found}\""))
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
