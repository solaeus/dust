//! Lexing tools.
//!
//! This module provides two lexing options:
//! - [`lex`], which lexes the entire input and returns a vector of tokens and their positions
//! - [`Lexer`], which lexes the input a token at a time
use std::num::{ParseFloatError, ParseIntError};

use crate::{Identifier, ReservedIdentifier, Span, Token};

/// Lex the input and return a vector of tokens and their positions.
pub fn lex(input: &str) -> Result<Vec<(Token, Span)>, LexError> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    loop {
        let (token, span) = lexer.next_token()?;
        let is_eof = matches!(token, Token::Eof);

        tokens.push((token, span));

        if is_eof {
            break;
        }
    }

    Ok(tokens)
}

#[derive(Debug, Clone)]
/// Low-level tool for lexing a single token at a time.
pub struct Lexer<'a> {
    source: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given input.
    pub fn new(input: &'a str) -> Self {
        Lexer {
            source: input,
            position: 0,
        }
    }

    /// Progress to the next character.
    fn next_char(&mut self) -> Option<char> {
        self.source[self.position..].chars().next().map(|c| {
            self.position += c.len_utf8();
            c
        })
    }

    /// Produce the next token.
    pub fn next_token(&mut self) -> Result<(Token, Span), LexError> {
        self.skip_whitespace();

        let (token, span) = if let Some(c) = self.peek_char() {
            match c {
                '0'..='9' => self.lex_number()?,
                'a'..='z' | 'A'..='Z' => self.lex_identifier()?,
                '+' => {
                    self.position += 1;
                    (Token::Plus, (self.position - 1, self.position))
                }
                '*' => {
                    self.position += 1;
                    (Token::Star, (self.position - 1, self.position))
                }
                '(' => {
                    self.position += 1;
                    (Token::LeftParenthesis, (self.position - 1, self.position))
                }
                ')' => {
                    self.position += 1;
                    (Token::RightParenthesis, (self.position - 1, self.position))
                }
                '=' => {
                    self.position += 1;
                    (Token::Equal, (self.position - 1, self.position))
                }
                '[' => {
                    self.position += 1;
                    (Token::LeftSquareBrace, (self.position - 1, self.position))
                }
                ']' => {
                    self.position += 1;
                    (Token::RightSquareBrace, (self.position - 1, self.position))
                }
                ',' => {
                    self.position += 1;
                    (Token::Comma, (self.position - 1, self.position))
                }
                '.' => {
                    self.position += 1;
                    (Token::Dot, (self.position - 1, self.position))
                }
                _ => (Token::Eof, (self.position, self.position)),
            }
        } else {
            (Token::Eof, (self.position, self.position))
        };

        Ok((token, span))
    }

    /// Skip whitespace characters.
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    /// Peek at the next character without consuming it.
    fn peek_char(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }

    /// Lex an integer or float token.
    fn lex_number(&mut self) -> Result<(Token, Span), LexError> {
        let start_pos = self.position;
        let mut is_float = false;

        while let Some(c) = self.peek_char() {
            if c == '.' {
                self.next_char();

                while let Some(c) = self.peek_char() {
                    if c.is_ascii_digit() {
                        is_float = true;

                        self.next_char();
                    } else {
                        break;
                    }
                }
            }

            if c.is_ascii_digit() {
                self.next_char();
            } else {
                break;
            }
        }

        if is_float {
            let float = self.source[start_pos..self.position].parse::<f64>()?;

            Ok((Token::Float(float), (start_pos, self.position)))
        } else {
            let integer = self.source[start_pos..self.position].parse::<i64>()?;

            Ok((Token::Integer(integer), (start_pos, self.position)))
        }
    }

    /// Lex an identifier token.
    fn lex_identifier(&mut self) -> Result<(Token, Span), LexError> {
        let start_pos = self.position;

        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() {
                self.next_char();
            } else {
                break;
            }
        }

        let string = &self.source[start_pos..self.position];
        let token = match string {
            "length" => Token::ReservedIdentifier(ReservedIdentifier::Length),
            _ => Token::Identifier(Identifier::new(string)),
        };

        Ok((token, (start_pos, self.position)))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum LexError {
    FloatError(ParseFloatError),
    IntegerError(ParseIntError),
}

impl From<ParseFloatError> for LexError {
    fn from(error: std::num::ParseFloatError) -> Self {
        Self::FloatError(error)
    }
}

impl From<ParseIntError> for LexError {
    fn from(error: std::num::ParseIntError) -> Self {
        Self::IntegerError(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_identifier() {
        let input = "length";

        assert_eq!(
            lex(input),
            Ok(vec![
                (
                    Token::ReservedIdentifier(ReservedIdentifier::Length),
                    (0, 6)
                ),
                (Token::Eof, (6, 6)),
            ])
        )
    }

    #[test]
    fn square_braces() {
        let input = "[]";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::LeftSquareBrace, (0, 1)),
                (Token::RightSquareBrace, (1, 2)),
                (Token::Eof, (2, 2)),
            ])
        )
    }

    #[test]
    fn small_float() {
        let input = "1.23";

        assert_eq!(
            lex(input),
            Ok(vec![(Token::Float(1.23), (0, 4)), (Token::Eof, (4, 4)),])
        )
    }

    #[test]
    #[allow(clippy::excessive_precision)]
    fn big_float() {
        let input = "123456789.123456789";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Float(123456789.123456789), (0, 19)),
                (Token::Eof, (19, 19)),
            ])
        )
    }

    #[test]
    fn add() {
        let input = "1 + 2";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer(1), (0, 1)),
                (Token::Plus, (2, 3)),
                (Token::Integer(2), (4, 5)),
                (Token::Eof, (5, 5)),
            ])
        )
    }

    #[test]
    fn multiply() {
        let input = "1 * 2";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer(1), (0, 1)),
                (Token::Star, (2, 3)),
                (Token::Integer(2), (4, 5)),
                (Token::Eof, (5, 5)),
            ])
        )
    }

    #[test]
    fn add_and_multiply() {
        let input = "1 + 2 * 3";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer(1), (0, 1)),
                (Token::Plus, (2, 3)),
                (Token::Integer(2), (4, 5)),
                (Token::Star, (6, 7)),
                (Token::Integer(3), (8, 9)),
                (Token::Eof, (9, 9)),
            ])
        );
    }

    #[test]
    fn assignment() {
        let input = "a = 1 + 2 * 3";

        assert_eq!(
            lex(input,),
            Ok(vec![
                (Token::Identifier(Identifier::new("a")), (0, 1)),
                (Token::Equal, (2, 3)),
                (Token::Integer(1), (4, 5)),
                (Token::Plus, (6, 7)),
                (Token::Integer(2), (8, 9)),
                (Token::Star, (10, 11)),
                (Token::Integer(3), (12, 13)),
                (Token::Eof, (13, 13)),
            ])
        );
    }
}
