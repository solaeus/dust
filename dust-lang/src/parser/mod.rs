#[cfg(test)]
mod tests;

use std::{
    fmt::{self, Display, Formatter},
    mem,
    num::ParseIntError,
};

use crate::{
    dust_error::AnnotatedError, Chunk, ChunkError, DustError, Identifier, Instruction, LexError,
    Lexer, Span, Token, TokenKind, TokenOwned, Value,
};

pub fn parse(source: &str) -> Result<Chunk, DustError> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);

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
    pub fn new(mut lexer: Lexer<'src>) -> Self {
        let (current_token, current_position) =
            lexer.next_token().unwrap_or((Token::Eof, Span(0, 0)));

        log::trace!("Starting parser with token {current_token} at {current_position}");

        Parser {
            lexer,
            chunk: Chunk::new(),
            current_register: 0,
            current_token,
            current_position,
            previous_token: Token::Eof,
            previous_position: Span(0, 0),
        }
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

        log::trace!("Advancing to token {new_token} at {position}");

        self.previous_token = mem::replace(&mut self.current_token, new_token);
        self.previous_position = mem::replace(&mut self.current_position, position);

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
        self.chunk.push_code(instruction, position);
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
        if let Token::Boolean(text) = self.previous_token {
            let boolean = text.parse::<bool>().unwrap();
            let value = Value::boolean(boolean);

            self.emit_constant(value)?;
        }

        Ok(())
    }

    fn parse_byte(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        if let Token::Byte(text) = self.previous_token {
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
        if let Token::Character(character) = self.previous_token {
            let value = Value::character(character);

            self.emit_constant(value)?;
        }

        Ok(())
    }

    fn parse_float(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        if let Token::Float(text) = self.previous_token {
            let float = text.parse::<f64>().unwrap();
            let value = Value::float(float);

            self.emit_constant(value)?;
        }

        Ok(())
    }

    fn parse_integer(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        if let Token::Integer(text) = self.previous_token {
            let integer = text.parse::<i64>().unwrap();
            let value = Value::integer(integer);

            self.emit_constant(value)?;
        }

        Ok(())
    }

    fn parse_string(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        if let Token::String(text) = self.previous_token {
            let value = Value::string(text);

            self.emit_constant(value)?;
        }

        Ok(())
    }

    fn parse_grouped(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        self.parse_expression()?;
        self.expect(TokenKind::RightParenthesis)
    }

    fn parse_unary(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        let operator_position = self.previous_position;
        let byte = match self.previous_token.kind() {
            TokenKind::Minus => {
                Instruction::negate(self.current_register, self.current_register - 1)
            }
            _ => {
                return Err(ParseError::ExpectedTokenMultiple {
                    expected: vec![TokenKind::Minus],
                    found: self.previous_token.to_owned(),
                    position: operator_position,
                })
            }
        };

        self.increment_register()?;
        self.parse_expression()?;
        self.emit_instruction(byte, operator_position);

        Ok(())
    }

    fn parse_binary(&mut self) -> Result<(), ParseError> {
        log::trace!("Parsing binary expression");

        let operator_position = self.previous_position;
        let operator = self.previous_token.kind();
        let rule = ParseRule::from(&operator);

        self.parse(rule.precedence.increment())?;

        let to_register = if self.current_register < 2 {
            self.current_register + 2
        } else {
            self.current_register
        };
        let left_register = to_register - 2;
        let right_register = to_register - 1;
        let byte = match operator {
            TokenKind::Plus => Instruction::add(to_register, left_register, right_register),
            TokenKind::Minus => Instruction::subtract(to_register, left_register, right_register),
            TokenKind::Star => Instruction::multiply(to_register, left_register, right_register),
            TokenKind::Slash => Instruction::divide(to_register, left_register, right_register),
            _ => {
                return Err(ParseError::ExpectedTokenMultiple {
                    expected: vec![
                        TokenKind::Plus,
                        TokenKind::Minus,
                        TokenKind::Star,
                        TokenKind::Slash,
                    ],
                    found: self.previous_token.to_owned(),
                    position: operator_position,
                })
            }
        };

        self.increment_register()?;
        self.emit_instruction(byte, operator_position);

        Ok(())
    }

    fn parse_variable(&mut self, allow_assignment: bool) -> Result<(), ParseError> {
        self.parse_named_variable(allow_assignment)
    }

    fn parse_named_variable(&mut self, allow_assignment: bool) -> Result<(), ParseError> {
        let token = self.previous_token.to_owned();
        let identifier_index = self.parse_identifier_from(token, self.previous_position)?;

        if allow_assignment && self.allow(TokenKind::Equal)? {
            self.parse_expression()?;

            self.emit_instruction(
                Instruction::set_variable(self.current_register, identifier_index),
                self.previous_position,
            );
            self.increment_register()?;
        } else {
            self.emit_instruction(
                Instruction::get_variable(self.current_register - 1, identifier_index),
                self.previous_position,
            );
        }

        Ok(())
    }

    fn parse_identifier_from(
        &mut self,
        token: TokenOwned,
        position: Span,
    ) -> Result<u16, ParseError> {
        if let TokenOwned::Identifier(text) = token {
            let identifier = Identifier::new(text);

            if let Ok(identifier_index) = self.chunk.get_identifier_index(&identifier, position) {
                Ok(identifier_index)
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

    pub fn parse_block(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        self.chunk.begin_scope();

        while !self.allow(TokenKind::RightCurlyBrace)? && !self.is_eof() {
            self.parse_statement()?;
        }

        self.chunk.end_scope();
        self.emit_instruction(
            Instruction::close(self.current_register),
            self.current_position,
        );

        Ok(())
    }

    fn parse_expression(&mut self) -> Result<(), ParseError> {
        self.parse(Precedence::None)
    }

    fn parse_statement(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;
        let (is_expression_statement, contains_block) = match self.current_token {
            Token::Let => {
                self.parse_let_statement(true)?;

                (false, false)
            }
            Token::LeftCurlyBrace => {
                self.parse_expression()?;

                (true, true)
            }
            _ => {
                self.parse_expression()?;

                (true, false)
            }
        };
        let has_semicolon = self.allow(TokenKind::Semicolon)?;

        if is_expression_statement && !contains_block && !has_semicolon {
            let end = self.previous_position.1;

            self.emit_instruction(Instruction::r#return(), Span(start, end))
        }

        Ok(())
    }

    fn parse_let_statement(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        self.expect(TokenKind::Let)?;

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

        let identifier_index = self.chunk.declare_variable(identifier, position)?;

        self.emit_instruction(
            Instruction::declare_variable(self.current_register, identifier_index),
            position,
        );
        self.increment_register()?;

        Ok(())
    }

    fn parse(&mut self, precedence: Precedence) -> Result<(), ParseError> {
        self.advance()?;

        let prefix_parser =
            if let Some(prefix) = ParseRule::from(&self.previous_token.kind()).prefix {
                log::trace!(
                    "Parsing {} as prefix with precedence {precedence}",
                    self.previous_token,
                );

                prefix
            } else {
                return Err(ParseError::ExpectedExpression {
                    found: self.previous_token.to_owned(),
                    position: self.previous_position,
                });
            };
        let allow_assignment = precedence <= Precedence::Assignment;

        prefix_parser(self, allow_assignment)?;

        while precedence < ParseRule::from(&self.current_token.kind()).precedence {
            self.advance()?;

            if let Some(infix_parser) = ParseRule::from(&self.previous_token.kind()).infix {
                log::trace!(
                    "Parsing {} as infix with precedence {precedence}",
                    self.previous_token,
                );

                if allow_assignment && self.current_token == Token::Equal {
                    return Err(ParseError::InvalidAssignmentTarget {
                        found: self.previous_token.to_owned(),
                        position: self.previous_position,
                    });
                }

                infix_parser(self)?;
            } else {
                break;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None = 0,
    Assignment = 1,
    Conditional = 2,
    LogicalOr = 3,
    LogicalAnd = 4,
    Equality = 5,
    Comparison = 6,
    Term = 7,
    Factor = 8,
    Unary = 9,
    Call = 10,
    Primary = 11,
}

impl Precedence {
    fn from_byte(byte: u8) -> Self {
        match byte {
            0 => Self::None,
            1 => Self::Assignment,
            2 => Self::Conditional,
            3 => Self::LogicalOr,
            4 => Self::LogicalAnd,
            5 => Self::Equality,
            6 => Self::Comparison,
            7 => Self::Term,
            8 => Self::Factor,
            9 => Self::Unary,
            10 => Self::Call,
            _ => Self::Primary,
        }
    }

    fn increment(&self) -> Self {
        Self::from_byte(*self as u8 + 1)
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
            TokenKind::Bang => todo!(),
            TokenKind::Colon => todo!(),
            TokenKind::Comma => todo!(),
            TokenKind::Dot => todo!(),
            TokenKind::DoubleAmpersand => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary),
                precedence: Precedence::LogicalAnd,
            },
            TokenKind::DoubleDot => todo!(),
            TokenKind::DoubleEqual => todo!(),
            TokenKind::DoublePipe => todo!(),
            TokenKind::Equal => todo!(),
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
            TokenKind::LeftSquareBrace => todo!(),
            TokenKind::Less => todo!(),
            TokenKind::LessOrEqual => todo!(),
            TokenKind::Minus => ParseRule {
                prefix: Some(Parser::parse_unary),
                infix: Some(Parser::parse_binary),
                precedence: Precedence::Term,
            },
            TokenKind::MinusEqual => todo!(),
            TokenKind::Mut => todo!(),
            TokenKind::Percent => todo!(),
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
            TokenKind::RightSquareBrace => todo!(),
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
        expected: Vec<TokenKind>,
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

    // Wrappers around foreign errors
    Chunk(ChunkError),
    Lex(LexError),
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
            Self::Chunk { .. } => "Chunk error",
            Self::Lex(_) => "Lex error",
            Self::ParseIntError { .. } => "Failed to parse integer",
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
            Self::Chunk(error) => error.details(),
            Self::Lex(error) => error.details(),
            Self::ParseIntError { error, .. } => Some(error.to_string()),
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
            Self::Chunk(error) => error.position(),
            Self::Lex(error) => error.position(),
            Self::ParseIntError { position, .. } => *position,
        }
    }
}

impl From<LexError> for ParseError {
    fn from(error: LexError) -> Self {
        Self::Lex(error)
    }
}
