//! Lexing tools.
//!
//! This module provides two lexing options:
//! - [`lex`], which lexes the entire input and returns a vector of tokens and their positions
//! - [`Lexer`], which lexes the input a token at a time
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    num::{ParseFloatError, ParseIntError},
};

use crate::{Span, Token};

/// Lexes the input and return a vector of tokens and their positions.
///
/// # Examples
/// ```
/// # use dust_lang::*;
/// let input = "x = 1 + 2";
/// let tokens = lex(input).unwrap();
///
/// assert_eq!(
///     tokens,
///     [
///         (Token::Identifier("x"), (0, 1)),
///         (Token::Equal, (2, 3)),
///         (Token::Integer(1), (4, 5)),
///         (Token::Plus, (6, 7)),
///         (Token::Integer(2), (8, 9)),
///         (Token::Eof, (9, 9)),
///     ]
/// );
/// ```
pub fn lex<'chars, 'src: 'chars>(input: &'src str) -> Result<Vec<(Token<'chars>, Span)>, LexError> {
    let mut lexer = Lexer::new();
    let mut tokens = Vec::new();

    loop {
        let (token, span) = lexer.next_token(input)?;
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
///
/// **Note**: It is a logic error to call `next_token` with different inputs.
///
/// # Examples
/// ```
/// # use dust_lang::*;
/// let input = "x = 1 + 2";
/// let mut lexer = Lexer::new();
/// let mut tokens = Vec::new();
///
/// loop {
///     let (token, span) = lexer.next_token(input).unwrap();
///     let is_eof = matches!(token, Token::Eof);
///
///     tokens.push((token, span));
///
///     if is_eof {
///         break;
///     }
/// }
///
/// assert_eq!(
///     tokens,
///     [
///         (Token::Identifier("x"), (0, 1)),
///         (Token::Equal, (2, 3)),
///         (Token::Integer(1), (4, 5)),
///         (Token::Plus, (6, 7)),
///         (Token::Integer(2), (8, 9)),
///         (Token::Eof, (9, 9)),
///     ]
/// )
/// ```
pub struct Lexer {
    position: usize,
}

impl Lexer {
    /// Create a new lexer for the given input.
    pub fn new() -> Self {
        Lexer { position: 0 }
    }

    /// Produce the next token.
    ///
    /// It is a logic error to call this method with different inputs.
    pub fn next_token<'src>(&mut self, source: &'src str) -> Result<(Token<'src>, Span), LexError> {
        self.skip_whitespace(source);

        let (token, span) = if let Some(c) = self.peek_char(source) {
            match c {
                '0'..='9' => self.lex_number(source)?,
                '-' => {
                    if let Some('0'..='9') = self.peek_second_char(source) {
                        self.lex_number(source)?
                    } else if "-Infinity" == self.peek_chars(source, 9) {
                        self.position += 9;
                        (
                            Token::Float(f64::NEG_INFINITY),
                            (self.position - 9, self.position),
                        )
                    } else {
                        self.position += 1;
                        (Token::Minus, (self.position - 1, self.position))
                    }
                }
                'a'..='z' | 'A'..='Z' => self.lex_alphabetical(source)?,
                '"' => self.lex_string('"', source)?,
                '\'' => self.lex_string('\'', source)?,
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
                _ => return Err(LexError::UnexpectedCharacter(c)),
            }
        } else {
            (Token::Eof, (self.position, self.position))
        };

        Ok((token, span))
    }

    /// Progress to the next character.
    fn next_char(&mut self, source: &str) -> Option<char> {
        source[self.position..].chars().next().map(|c| {
            self.position += c.len_utf8();
            c
        })
    }

    /// Skip whitespace characters.
    fn skip_whitespace(&mut self, source: &str) {
        while let Some(c) = self.peek_char(source) {
            if c.is_whitespace() {
                self.next_char(source);
            } else {
                break;
            }
        }
    }

    /// Peek at the next character without consuming it.
    fn peek_char(&self, source: &str) -> Option<char> {
        source[self.position..].chars().next()
    }

    /// Peek at the second-to-next character without consuming it.
    fn peek_second_char(&self, source: &str) -> Option<char> {
        source[self.position..].chars().nth(1)
    }

    /// Peek the next `n` characters without consuming them.
    fn peek_chars<'src>(&self, source: &'src str, n: usize) -> &'src str {
        let remaining_source = &source[self.position..];

        if remaining_source.len() < n {
            remaining_source
        } else {
            &remaining_source[..n]
        }
    }

    /// Lex an integer or float token.
    fn lex_number<'src>(&mut self, source: &'src str) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;
        let mut is_float = false;

        if let Some('-') = self.peek_char(source) {
            self.next_char(source);
        }

        while let Some(c) = self.peek_char(source) {
            if c == '.' {
                if let Some('0'..='9') = self.peek_second_char(source) {
                    if !is_float {
                        self.next_char(source);
                    }

                    self.next_char(source);

                    loop {
                        let peek_char = self.peek_char(source);

                        if let Some('0'..='9') = peek_char {
                            self.next_char(source);
                        } else if let Some('e') = peek_char {
                            if let Some('0'..='9') = self.peek_second_char(source) {
                                self.next_char(source);
                                self.next_char(source);
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    is_float = true;
                } else {
                    break;
                }
            }

            if c.is_ascii_digit() {
                self.next_char(source);
            } else {
                break;
            }
        }

        if is_float {
            let float = source[start_pos..self.position].parse::<f64>()?;

            Ok((Token::Float(float), (start_pos, self.position)))
        } else {
            let integer = source[start_pos..self.position].parse::<i64>()?;

            Ok((Token::Integer(integer), (start_pos, self.position)))
        }
    }

    /// Lex an identifier token.
    fn lex_alphabetical<'src>(
        &mut self,
        source: &'src str,
    ) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        while let Some(c) = self.peek_char(source) {
            if c.is_ascii_alphabetic() || c == '_' {
                self.next_char(source);
            } else {
                break;
            }
        }

        let string = &source[start_pos..self.position];
        let token = match string {
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),
            "Infinity" => Token::Float(f64::INFINITY),
            "is_even" => Token::IsEven,
            "is_odd" => Token::IsOdd,
            "length" => Token::Length,
            "NaN" => Token::Float(f64::NAN),
            "read_line" => Token::ReadLine,
            "write_line" => Token::WriteLine,
            _ => Token::Identifier(string),
        };

        Ok((token, (start_pos, self.position)))
    }

    fn lex_string<'src>(
        &mut self,
        delimiter: char,
        source: &'src str,
    ) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char(source);

        while let Some(c) = self.peek_char(source) {
            if c == delimiter {
                self.next_char(source);
                break;
            } else {
                self.next_char(source);
            }
        }

        let text = &source[start_pos + 1..self.position - 1];

        Ok((Token::String(text), (start_pos, self.position)))
    }
}

impl Default for Lexer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum LexError {
    FloatError(ParseFloatError),
    IntegerError(ParseIntError),
    UnexpectedCharacter(char),
}

impl Error for LexError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FloatError(parse_float_error) => Some(parse_float_error),
            Self::IntegerError(parse_int_error) => Some(parse_int_error),
            Self::UnexpectedCharacter(_) => None,
        }
    }
}

impl Display for LexError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::FloatError(parse_float_error) => {
                write!(f, "Failed to parse float: {}", parse_float_error)
            }
            Self::IntegerError(parse_int_error) => {
                write!(f, "Failed to parse integer: {}", parse_int_error)
            }
            Self::UnexpectedCharacter(character) => {
                write!(f, "Unexpected character: '{}'", character)
            }
        }
    }
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
    fn infinity() {
        let input = "Infinity";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Float(f64::INFINITY), (0, 8)),
                (Token::Eof, (8, 8)),
            ])
        )
    }

    #[test]
    fn negative_infinity() {
        let input = "-Infinity";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Float(f64::NEG_INFINITY), (0, 9)),
                (Token::Eof, (9, 9)),
            ])
        )
    }

    #[test]
    fn nan() {
        let input = "NaN";

        assert!(lex(input).is_ok_and(|tokens| tokens[0].0 == Token::Float(f64::NAN)));
    }

    #[test]
    fn complex_float() {
        let input = "42.42e42";

        assert_eq!(
            lex(input),
            Ok(vec![(Token::Float(42.42e42), (0, 8)), (Token::Eof, (8, 8)),])
        )
    }

    #[test]
    fn max_integer() {
        let input = "9223372036854775807";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer(i64::MAX), (0, 19)),
                (Token::Eof, (19, 19)),
            ])
        )
    }

    #[test]
    fn min_integer() {
        let input = "-9223372036854775808";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer(i64::MIN), (0, 20)),
                (Token::Eof, (20, 20)),
            ])
        )
    }

    #[test]
    fn subtract_negative_integers() {
        let input = "-42 - -42";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer(-42), (0, 3)),
                (Token::Minus, (4, 5)),
                (Token::Integer(-42), (6, 9)),
                (Token::Eof, (9, 9)),
            ])
        )
    }

    #[test]
    fn negative_integer() {
        let input = "-42";

        assert_eq!(
            lex(input),
            Ok(vec![(Token::Integer(-42), (0, 3)), (Token::Eof, (3, 3))])
        )
    }

    #[test]
    fn read_line() {
        let input = "read_line()";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::ReadLine, (0, 9)),
                (Token::LeftParenthesis, (9, 10)),
                (Token::RightParenthesis, (10, 11)),
                (Token::Eof, (11, 11)),
            ])
        )
    }

    #[test]
    fn write_line() {
        let input = "write_line('Hello, world!')";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::WriteLine, (0, 10)),
                (Token::LeftParenthesis, (10, 11)),
                (Token::String("Hello, world!"), (11, 26)),
                (Token::RightParenthesis, (26, 27)),
                (Token::Eof, (27, 27)),
            ])
        )
    }

    #[test]
    fn string_concatenation() {
        let input = "'Hello, ' + 'world!'";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::String("Hello, "), (0, 9)),
                (Token::Plus, (10, 11)),
                (Token::String("world!"), (12, 20)),
                (Token::Eof, (20, 20)),
            ])
        )
    }

    #[test]
    fn string() {
        let input = "'Hello, world!'";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::String("Hello, world!"), (0, 15)),
                (Token::Eof, (15, 15)),
            ])
        )
    }

    #[test]
    fn r#true() {
        let input = "true";

        assert_eq!(
            lex(input),
            Ok(vec![(Token::Boolean(true), (0, 4)), (Token::Eof, (4, 4)),])
        )
    }

    #[test]
    fn r#false() {
        let input = "false";

        assert_eq!(
            lex(input),
            Ok(vec![(Token::Boolean(false), (0, 5)), (Token::Eof, (5, 5))])
        )
    }

    #[test]
    fn property_access_function_call() {
        let input = "42.is_even()";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer(42), (0, 2)),
                (Token::Dot, (2, 3)),
                (Token::IsEven, (3, 10)),
                (Token::LeftParenthesis, (10, 11)),
                (Token::RightParenthesis, (11, 12)),
                (Token::Eof, (12, 12)),
            ])
        )
    }

    #[test]
    fn empty() {
        let input = "";

        assert_eq!(lex(input), Ok(vec![(Token::Eof, (0, 0))]))
    }

    #[test]
    fn reserved_identifier() {
        let input = "length";

        assert_eq!(
            lex(input),
            Ok(vec![(Token::Length, (0, 6)), (Token::Eof, (6, 6)),])
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
                (Token::Identifier("a"), (0, 1)),
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
