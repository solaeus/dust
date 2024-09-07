use std::{
    fmt::{self, Display, Formatter},
    num::ParseIntError,
};

use crate::{
    Chunk, ChunkError, Instruction, LexError, Lexer, Span, Token, TokenKind, TokenOwned, Value,
};

pub fn parse(source: &str) -> Result<Chunk, ParseError> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);

    while !parser.is_eof() {
        parser.parse(Precedence::None)?;
    }

    Ok(parser.chunk)
}

#[derive(Debug)]
pub struct Parser<'src> {
    lexer: Lexer<'src>,
    chunk: Chunk,
    current_token: Option<Token<'src>>,
    current_position: Span,
}

impl<'src> Parser<'src> {
    pub fn new(lexer: Lexer<'src>) -> Self {
        Parser {
            lexer,
            chunk: Chunk::new(),
            current_token: None,
            current_position: Span(0, 0),
        }
    }

    fn is_eof(&self) -> bool {
        matches!(self.current_token, Some(Token::Eof))
    }

    fn advance(&mut self) -> Result<(), ParseError> {
        let (token, position) = self.lexer.next_token()?;

        log::trace!("Advancing to token {token} at {position}");

        self.current_token = Some(token);
        self.current_position = position;

        Ok(())
    }

    fn current_token_owned(&self) -> TokenOwned {
        self.current_token
            .as_ref()
            .map_or(TokenOwned::Eof, |token| token.to_owned())
    }

    fn current_token_kind(&self) -> TokenKind {
        self.current_token
            .as_ref()
            .map_or(TokenKind::Eof, |token| token.kind())
    }

    fn consume(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        if self.current_token_kind() == expected {
            self.advance()
        } else {
            Err(ParseError::ExpectedToken {
                expected,
                found: self.current_token_owned(),
                position: self.current_position,
            })
        }
    }

    fn emit_byte(&mut self, byte: u8, position: Span) {
        self.chunk.write(byte, position);
    }

    fn emit_constant(&mut self, value: Value) -> Result<(), ParseError> {
        let constant_index = self.chunk.push_constant(value)?;
        let position = self.current_position;

        self.emit_byte(Instruction::Constant as u8, position);
        self.emit_byte(constant_index, position);

        Ok(())
    }

    fn parse_integer(&mut self) -> Result<(), ParseError> {
        if let Some(Token::Integer(text)) = self.current_token {
            let integer = text.parse::<i64>().unwrap();
            let value = Value::integer(integer);

            self.emit_constant(value)?;
        }

        Ok(())
    }

    fn parse_grouped(&mut self) -> Result<(), ParseError> {
        self.parse_expression()?;

        self.consume(TokenKind::RightParenthesis)?;

        Ok(())
    }

    fn parse_unary(&mut self) -> Result<(), ParseError> {
        if let Some(Token::Minus) = self.current_token {
            let operator_position = self.current_position;

            self.advance()?;
            self.parse_expression()?;
            self.emit_byte(Instruction::Negate as u8, operator_position);
        }

        Ok(())
    }

    fn parse_binary(&mut self) -> Result<(), ParseError> {
        let operator_position = self.current_position;
        let operator = self.current_token_kind();
        let rule = ParseRule::from(&operator);

        self.parse(rule.precedence.increment())?;

        let byte = match operator {
            TokenKind::Plus => Instruction::Add as u8,
            TokenKind::Minus => Instruction::Subtract as u8,
            TokenKind::Star => Instruction::Multiply as u8,
            TokenKind::Slash => Instruction::Divide as u8,
            _ => {
                return Err(ParseError::ExpectedTokenMultiple {
                    expected: vec![
                        TokenKind::Plus,
                        TokenKind::Minus,
                        TokenKind::Star,
                        TokenKind::Slash,
                    ],
                    found: self.current_token_owned(),
                    position: self.current_position,
                })
            }
        };

        self.emit_byte(byte, operator_position);

        Ok(())
    }

    fn parse_expression(&mut self) -> Result<(), ParseError> {
        self.parse(Precedence::Assignment)
    }

    // Pratt parsing functions

    fn parse(&mut self, precedence: Precedence) -> Result<(), ParseError> {
        log::trace!("Parsing with precedence {precedence}");

        self.advance()?;

        let prefix_rule = ParseRule::from(&self.current_token_kind()).prefix;

        if let Some(prefix) = prefix_rule {
            log::trace!("Parsing {} as prefix", &self.current_token_owned());

            prefix(self)?;
        } else {
            return Err(ParseError::ExpectedPrefix {
                found: self.current_token_owned(),
                position: self.current_position,
            });
        }

        while precedence <= ParseRule::from(&self.current_token_kind()).precedence {
            self.advance()?;

            let infix_rule = ParseRule::from(&self.current_token_kind()).infix;

            if let Some(infix) = infix_rule {
                log::trace!("Parsing {} as infix", self.current_token_owned());

                infix(self)?;
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

    fn decrement(&self) -> Self {
        Self::from_byte(*self as u8 - 1)
    }
}

impl Display for Precedence {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

type ParserFunction<'a> = fn(&'_ mut Parser<'a>) -> Result<(), ParseError>;

#[derive(Debug, Clone, Copy)]
pub struct ParseRule<'a> {
    pub prefix: Option<ParserFunction<'a>>,
    pub infix: Option<ParserFunction<'a>>,
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
            TokenKind::Identifier => todo!(),
            TokenKind::Boolean => todo!(),
            TokenKind::Character => todo!(),
            TokenKind::Float => todo!(),
            TokenKind::Integer => ParseRule {
                prefix: Some(Parser::parse_integer),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::String => todo!(),
            TokenKind::Async => todo!(),
            TokenKind::Bool => todo!(),
            TokenKind::Break => todo!(),
            TokenKind::Else => todo!(),
            TokenKind::FloatKeyword => todo!(),
            TokenKind::If => todo!(),
            TokenKind::Int => todo!(),
            TokenKind::Let => todo!(),
            TokenKind::Loop => todo!(),
            TokenKind::Map => todo!(),
            TokenKind::Str => todo!(),
            TokenKind::While => todo!(),
            TokenKind::BangEqual => todo!(),
            TokenKind::Bang => todo!(),
            TokenKind::Colon => todo!(),
            TokenKind::Comma => todo!(),
            TokenKind::Dot => todo!(),
            TokenKind::DoubleAmpersand => todo!(),
            TokenKind::DoubleDot => todo!(),
            TokenKind::DoubleEqual => todo!(),
            TokenKind::DoublePipe => todo!(),
            TokenKind::Equal => todo!(),
            TokenKind::Greater => todo!(),
            TokenKind::GreaterOrEqual => todo!(),
            TokenKind::LeftCurlyBrace => todo!(),
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
            TokenKind::RightCurlyBrace => todo!(),
            TokenKind::RightParenthesis => todo!(),
            TokenKind::RightSquareBrace => todo!(),
            TokenKind::Semicolon => todo!(),
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
    ExpectedPrefix {
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

    // Wrappers around foreign errors
    Chunk(ChunkError),
    Lex(LexError),
    ParseIntError(ParseIntError),
}

impl From<ParseIntError> for ParseError {
    fn from(error: ParseIntError) -> Self {
        Self::ParseIntError(error)
    }
}

impl From<LexError> for ParseError {
    fn from(error: LexError) -> Self {
        Self::Lex(error)
    }
}

impl From<ChunkError> for ParseError {
    fn from(error: ChunkError) -> Self {
        Self::Chunk(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_integer() {
        let source = "42";
        let test_chunk = parse(source);

        assert_eq!(
            test_chunk,
            Ok(Chunk::with_data(
                vec![(Instruction::Constant as u8, Span(0, 2)), (0, Span(0, 2))],
                vec![Value::integer(42)]
            ))
        );
    }

    #[test]
    fn parse_addition() {
        env_logger::builder().is_test(true).try_init().unwrap();

        let source = "42 + 42";
        let test_chunk = parse(source);

        assert_eq!(
            test_chunk,
            Ok(Chunk::with_data(
                vec![
                    (Instruction::Constant as u8, Span(0, 2)),
                    (0, Span(0, 2)),
                    (Instruction::Constant as u8, Span(5, 7)),
                    (1, Span(5, 7)),
                    (Instruction::Add as u8, Span(3, 4)),
                ],
                vec![Value::integer(42), Value::integer(42)]
            ))
        );
    }
}
