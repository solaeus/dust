#[cfg(test)]
mod tests;

use std::{
    fmt::{self, Display, Formatter},
    mem::replace,
    num::{ParseFloatError, ParseIntError},
};

use crate::{
    dust_error::AnnotatedError, Chunk, ChunkError, DustError, Identifier, Instruction, LexError,
    Lexer, Operation, Span, Token, TokenKind, TokenOwned, Value,
};

pub fn parse(source: &str) -> Result<Chunk, DustError> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer).map_err(|error| DustError::Parse { error, source })?;

    while !parser.is_eof() {
        parser
            .parse_statement()
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

    fn parse_boolean(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        if let Token::Boolean(text) = self.current_token {
            self.advance()?;

            let boolean = text.parse::<bool>().unwrap();

            self.emit_instruction(
                Instruction::load_boolean(self.current_register, boolean, false),
                self.previous_position,
            );
        }

        Ok(())
    }

    fn parse_byte(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
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

    fn parse_character(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        if let Token::Character(character) = self.current_token {
            self.advance()?;

            let value = Value::character(character);

            self.emit_constant(value)?;
        }

        Ok(())
    }

    fn parse_float(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
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

    fn parse_integer(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
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

    fn parse_string(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        if let Token::String(text) = self.current_token {
            self.advance()?;

            let value = Value::string(text);

            self.emit_constant(value)?;
        }

        Ok(())
    }

    fn parse_grouped(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        self.allow(TokenKind::LeftParenthesis)?;
        self.parse_expression()?;
        self.expect(TokenKind::RightParenthesis)
    }

    fn parse_unary(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
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
        let (left_instruction, left_position) =
            self.chunk.pop_instruction(self.current_position)?;
        let mut push_back_left = false;
        let mut left_is_constant = false;
        let left = match left_instruction.operation() {
            Operation::LoadConstant => {
                log::trace!(
                    "Condensing {} to binary expression",
                    left_instruction.operation()
                );

                left_is_constant = true;

                self.decrement_register()?;
                left_instruction.first_argument()
            }
            Operation::GetLocal => {
                log::trace!(
                    "Condensing {} to binary expression",
                    left_instruction.operation()
                );

                self.decrement_register()?;
                left_instruction.first_argument()
            }
            Operation::Close => {
                return Err(ParseError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                });
            }
            _ => {
                push_back_left = true;

                left_instruction.destination()
            }
        };

        let operator = self.current_token;
        let operator_position = self.current_position;
        let rule = ParseRule::from(&operator.kind());

        self.advance()?;
        self.parse(rule.precedence.increment())?;

        let (right_instruction, right_position) =
            self.chunk.pop_instruction(self.current_position)?;
        let mut push_back_right = false;
        let mut right_is_constant = false;
        let right = match right_instruction.operation() {
            Operation::LoadConstant => {
                log::trace!(
                    "Condensing {} to binary expression",
                    right_instruction.operation()
                );

                right_is_constant = true;

                self.decrement_register()?;
                right_instruction.first_argument()
            }
            Operation::GetLocal => {
                log::trace!(
                    "Condensing {} to binary expression",
                    right_instruction.operation()
                );

                self.decrement_register()?;
                right_instruction.first_argument()
            }
            Operation::Close => {
                return Err(ParseError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                });
            }
            _ => {
                push_back_right = true;

                right_instruction.destination()
            }
        };

        let mut instruction = match operator.kind() {
            TokenKind::Plus => Instruction::add(self.current_register, left, right),
            TokenKind::Minus => Instruction::subtract(self.current_register, left, right),
            TokenKind::Star => Instruction::multiply(self.current_register, left, right),
            TokenKind::Slash => Instruction::divide(self.current_register, left, right),
            TokenKind::Percent => Instruction::modulo(self.current_register, left, right),
            TokenKind::DoubleAmpersand => Instruction::and(self.current_register, left, right),
            TokenKind::DoublePipe => Instruction::or(self.current_register, left, right),
            _ => {
                return Err(ParseError::ExpectedTokenMultiple {
                    expected: &[
                        TokenKind::Plus,
                        TokenKind::Minus,
                        TokenKind::Star,
                        TokenKind::Slash,
                        TokenKind::Percent,
                        TokenKind::DoubleAmpersand,
                        TokenKind::DoublePipe,
                    ],
                    found: operator.to_owned(),
                    position: operator_position,
                })
            }
        };

        if left_is_constant {
            instruction.set_first_argument_to_constant();
        }

        if right_is_constant {
            instruction.set_second_argument_to_constant();
        }

        if push_back_left {
            self.emit_instruction(left_instruction, left_position);
        }

        if push_back_right {
            self.emit_instruction(right_instruction, right_position);
        }

        self.emit_instruction(instruction, operator_position);
        self.increment_register()?;

        Ok(())
    }

    fn parse_variable(&mut self, allow_assignment: bool) -> Result<(), ParseError> {
        self.parse_named_variable(allow_assignment)
    }

    fn parse_named_variable(&mut self, allow_assignment: bool) -> Result<(), ParseError> {
        let token = self.current_token.to_owned();
        let start_position = self.current_position;
        let local_index = self.parse_identifier_from(token, start_position)?;

        self.advance()?;

        if allow_assignment && self.allow(TokenKind::Equal)? {
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
                } else {
                    self.emit_instruction(previous_instruction, previous_position);
                    self.emit_instruction(
                        Instruction::set_local(self.current_register - 1, local_index),
                        self.current_position,
                    );
                }
            } else {
                self.emit_instruction(previous_instruction, previous_position);
                self.emit_instruction(
                    Instruction::set_local(self.current_register - 1, local_index),
                    start_position,
                );
            }
        } else {
            self.emit_instruction(
                Instruction::get_local(self.current_register, local_index),
                self.previous_position,
            );
            self.increment_register()?;
        }

        Ok(())
    }

    fn parse_identifier_from(
        &mut self,
        token: TokenOwned,
        position: Span,
    ) -> Result<u8, ParseError> {
        if let TokenOwned::Identifier(text) = token {
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

    fn parse_block(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        self.advance()?;
        self.chunk.begin_scope();

        let start = self.current_position.0;
        let start_register = self.current_register;
        let mut ends_with_semicolon = false;

        while !self.allow(TokenKind::RightCurlyBrace)? && !self.is_eof() {
            self.parse_statement()?;

            if self.previous_token == Token::Semicolon {
                ends_with_semicolon = true;
            }
        }

        self.chunk.end_scope();

        if self.current_token == Token::Semicolon {
            ends_with_semicolon = true;
        }

        if ends_with_semicolon {
            let end = self.current_position.1;
            let end_register = self.current_register;

            self.emit_instruction(
                Instruction::close(start_register, end_register),
                Span(start, end),
            );
        }

        Ok(())
    }

    fn parse_list(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        let start = self.current_position.0;

        self.advance()?;

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
            Instruction::load_list(self.current_register, length),
            Span(start, end),
        );

        Ok(())
    }

    fn parse_expression(&mut self) -> Result<(), ParseError> {
        self.parse(Precedence::None)
    }

    fn parse_statement(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;
        let is_expression = match self.current_token {
            Token::Let => {
                self.parse_let_statement(true)?;

                false
            }
            Token::LeftCurlyBrace => {
                self.parse_block(true)?;

                self.previous_token != Token::Semicolon
            }
            _ => {
                self.parse_expression()?;

                true
            }
        };
        let has_semicolon = self.allow(TokenKind::Semicolon)?;

        if (!has_semicolon && is_expression) && self.is_eof() {
            let end = self.previous_position.1;

            self.emit_instruction(Instruction::r#return(), Span(start, end));
        }

        Ok(())
    }

    fn parse_let_statement(&mut self, allow_assignment: bool) -> Result<(), ParseError> {
        if !allow_assignment {
            return Err(ParseError::ExpectedExpression {
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        }

        self.advance()?;

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

        let register = self.chunk.get_last_instruction(position)?.0.destination();
        let local_index =
            self.chunk
                .declare_local(identifier, is_mutable, register, self.current_position)?;

        self.emit_instruction(
            Instruction::define_local(register, local_index, is_mutable),
            position,
        );
        self.allow(TokenKind::Semicolon)?;

        Ok(())
    }

    fn parse(&mut self, precedence: Precedence) -> Result<(), ParseError> {
        let allow_assignment = precedence < Precedence::Assignment;

        if let Some(prefix_parser) = ParseRule::from(&self.current_token.kind()).prefix {
            log::trace!(
                "Parsing \"{}\" with prefix parser at precedence {precedence}",
                self.current_token,
            );

            prefix_parser(self, allow_assignment)?;
        }

        let mut infix_rule = ParseRule::from(&self.current_token.kind());

        while precedence <= infix_rule.precedence {
            if let Some(infix_parser) = infix_rule.infix {
                log::trace!(
                    "Parsing \"{}\" with infix parser at precedence {precedence}",
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

type PrefixFunction<'a> = fn(&mut Parser<'a>, bool) -> Result<(), ParseError>;
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
            TokenKind::Eof => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Identifier => ParseRule {
                prefix: Some(Parser::parse_variable),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Boolean => ParseRule {
                prefix: Some(Parser::parse_boolean),
                infix: None,
                precedence: Precedence::None,
            },
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
            TokenKind::Float => ParseRule {
                prefix: Some(Parser::parse_float),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Integer => ParseRule {
                prefix: Some(Parser::parse_integer),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::String => ParseRule {
                prefix: Some(Parser::parse_string),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Async => todo!(),
            TokenKind::Bool => todo!(),
            TokenKind::Break => todo!(),
            TokenKind::Else => todo!(),
            TokenKind::FloatKeyword => todo!(),
            TokenKind::If => todo!(),
            TokenKind::Int => todo!(),
            TokenKind::Let => ParseRule {
                prefix: Some(Parser::parse_let_statement),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Loop => todo!(),
            TokenKind::Map => todo!(),
            TokenKind::Str => todo!(),
            TokenKind::While => todo!(),
            TokenKind::BangEqual => todo!(),
            TokenKind::Bang => ParseRule {
                prefix: Some(Parser::parse_unary),
                infix: None,
                precedence: Precedence::Unary,
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
            TokenKind::DoubleDot => todo!(),
            TokenKind::DoubleEqual => todo!(),
            TokenKind::DoublePipe => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary),
                precedence: Precedence::LogicalOr,
            },
            TokenKind::Equal => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::Assignment,
            },
            TokenKind::Greater => todo!(),
            TokenKind::GreaterOrEqual => todo!(),
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
            TokenKind::Less => todo!(),
            TokenKind::LessOrEqual => todo!(),
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
            TokenKind::Star => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary),
                precedence: Precedence::Factor,
            },
            TokenKind::Struct => todo!(),
            TokenKind::Slash => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary),
                precedence: Precedence::Factor,
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
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
            Self::ExpectedExpression { found, .. } => Some(format!("Found \"{found}\"")),
            Self::ExpectedToken {
                expected, found, ..
            } => Some(format!("Expected \"{expected}\", found \"{found}\"")),
            Self::ExpectedTokenMultiple {
                expected, found, ..
            } => Some(format!("Expected one of {expected:?}, found \"{found}\"")),
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
