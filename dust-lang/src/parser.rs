use std::num::ParseIntError;

use crate::{
    Chunk, ChunkError, Instruction, LexError, Lexer, Span, Token, TokenKind, TokenOwned, Value,
};

#[derive(Debug)]
pub struct Parser<'src> {
    lexer: Lexer<'src>,
    current_token: Token<'src>,
    current_position: Span,
}

impl<'src> Parser<'src> {
    pub fn new(mut lexer: Lexer<'src>) -> Self {
        let (current_token, current_position) =
            lexer.next_token().unwrap_or((Token::Eof, Span(0, 0)));

        Parser {
            lexer,
            current_token,
            current_position,
        }
    }

    fn is_eof(&self) -> bool {
        matches!(self.current_token, Token::Eof)
    }

    fn advance(&mut self) -> Result<(), ParseError> {
        let (token, position) = self.lexer.next_token()?;

        self.current_token = token;
        self.current_position = position;

        Ok(())
    }

    fn consume(&mut self, expected: TokenKind) -> Result<(), ParseError> {
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

    fn emit_instruction(&mut self, instruction: Instruction, chunk: &mut Chunk) {
        chunk.write(instruction as u8, self.current_position);
    }

    fn parse_prefix(&mut self, chunk: &mut Chunk) -> Result<(), ParseError> {
        Ok(())
    }

    fn parse_primary(&mut self, chunk: &mut Chunk) -> Result<(), ParseError> {
        match self.current_token {
            Token::Integer(text) => {
                let integer = text.parse::<i64>()?;
                let value = Value::integer(integer);
                let constant_index = chunk.push_constant(value)?;

                chunk.write(Instruction::Constant as u8, self.current_position);
                chunk.write(constant_index, self.current_position);
            }
            Token::LeftParenthesis => {}
            _ => {
                return Err(ParseError::ExpectedTokenMultiple {
                    expected: vec![TokenKind::Integer],
                    found: self.current_token.to_owned(),
                    position: self.current_position,
                })
            }
        }

        Ok(())
    }

    pub fn parse_postfix(&mut self, left: Value, chunk: &mut Chunk) -> Result<(), ParseError> {
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
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
