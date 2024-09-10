//! Lexing tools.
//!
//! This module provides two lexing options:
//! - [`lex`], which lexes the entire input and returns a vector of tokens and their positions
//! - [`Lexer`], which lexes the input a token at a time
use std::fmt::{self, Display, Formatter};

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
///         (Token::Identifier("x"), Span(0, 1)),
///         (Token::Equal, Span(2, 3)),
///         (Token::Integer("1"), Span(4, 5)),
///         (Token::Plus, Span(6, 7)),
///         (Token::Integer("2"), Span(8, 9)),
///         (Token::Eof, Span(9, 9)),
///     ]
/// );
/// ```
pub fn lex<'chars, 'src: 'chars>(
    source: &'src str,
) -> Result<Vec<(Token<'chars>, Span)>, LexError> {
    let mut lexer = Lexer::new(source);
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

/// Low-level tool for lexing a single token at a time.
///
/// # Examples
/// ```
/// # use dust_lang::*;
/// let input = "x = 1 + 2";
/// let mut lexer = Lexer::new(input);
/// let mut tokens = Vec::new();
///
/// loop {
///     let (token, span) = lexer.next_token().unwrap();
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
///         (Token::Identifier("x"), Span(0, 1)),
///         (Token::Equal, Span(2, 3)),
///         (Token::Integer("1"), Span(4, 5)),
///         (Token::Plus, Span(6, 7)),
///         (Token::Integer("2"), Span(8, 9)),
///         (Token::Eof, Span(9, 9)),
///     ]
/// )
/// ```
#[derive(Debug, Clone)]
pub struct Lexer<'src> {
    source: &'src str,
    position: usize,
}

impl<'src> Lexer<'src> {
    /// Create a new lexer for the given input.
    pub fn new(source: &'src str) -> Self {
        Lexer {
            source,
            position: 0,
        }
    }

    /// Produce the next token.
    pub fn next_token(&mut self) -> Result<(Token<'src>, Span), LexError> {
        self.skip_whitespace();

        let (token, span) = if let Some(c) = self.peek_char() {
            match c {
                '0'..='9' => self.lex_number()?,
                '-' => {
                    let second_char = self.peek_second_char();

                    if let Some('=') = second_char {
                        self.position += 2;

                        (Token::MinusEqual, Span(self.position - 2, self.position))
                    } else if let Some('0'..='9') = second_char {
                        self.lex_number()?
                    } else if "-Infinity" == self.peek_chars(9) {
                        self.position += 9;

                        (
                            Token::Float("-Infinity"),
                            Span(self.position - 9, self.position),
                        )
                    } else {
                        self.position += 1;

                        (Token::Minus, Span(self.position - 1, self.position))
                    }
                }
                'a'..='z' | 'A'..='Z' => self.lex_alphanumeric()?,
                '"' => self.lex_string()?,
                '\'' => {
                    self.position += 1;

                    if let Some(c) = self.peek_char() {
                        self.position += 1;

                        if let Some('\'') = self.peek_char() {
                            self.position += 1;

                            (Token::Character(c), Span(self.position - 3, self.position))
                        } else {
                            return Err(LexError::ExpectedCharacter {
                                expected: '\'',
                                actual: c,
                                position: self.position,
                            });
                        }
                    } else {
                        return Err(LexError::UnexpectedEndOfFile {
                            position: self.position,
                        });
                    }
                }
                '+' => {
                    if let Some('=') = self.peek_second_char() {
                        self.position += 2;

                        (Token::PlusEqual, Span(self.position - 2, self.position))
                    } else {
                        self.position += 1;

                        (Token::Plus, Span(self.position - 1, self.position))
                    }
                }
                '*' => {
                    self.position += 1;

                    (Token::Star, Span(self.position - 1, self.position))
                }
                '(' => {
                    self.position += 1;

                    (
                        Token::LeftParenthesis,
                        Span(self.position - 1, self.position),
                    )
                }
                ')' => {
                    self.position += 1;

                    (
                        Token::RightParenthesis,
                        Span(self.position - 1, self.position),
                    )
                }
                '=' => {
                    if let Some('=') = self.peek_second_char() {
                        self.position += 2;

                        (Token::DoubleEqual, Span(self.position - 2, self.position))
                    } else {
                        self.position += 1;

                        (Token::Equal, Span(self.position - 1, self.position))
                    }
                }
                '[' => {
                    self.position += 1;

                    (
                        Token::LeftSquareBrace,
                        Span(self.position - 1, self.position),
                    )
                }
                ']' => {
                    self.position += 1;

                    (
                        Token::RightSquareBrace,
                        Span(self.position - 1, self.position),
                    )
                }
                ',' => {
                    self.position += 1;

                    (Token::Comma, Span(self.position - 1, self.position))
                }
                '.' => {
                    if let Some('.') = self.peek_second_char() {
                        self.position += 2;

                        (Token::DoubleDot, Span(self.position - 2, self.position))
                    } else {
                        self.position += 1;

                        (Token::Dot, Span(self.position - 1, self.position))
                    }
                }
                '>' => {
                    if let Some('=') = self.peek_second_char() {
                        self.position += 2;

                        (Token::GreaterEqual, Span(self.position - 2, self.position))
                    } else {
                        self.position += 1;

                        (Token::Greater, Span(self.position - 1, self.position))
                    }
                }
                '<' => {
                    if let Some('=') = self.peek_second_char() {
                        self.position += 2;

                        (Token::LessEqual, Span(self.position - 2, self.position))
                    } else {
                        self.position += 1;

                        (Token::Less, Span(self.position - 1, self.position))
                    }
                }
                '{' => {
                    self.position += 1;

                    (
                        Token::LeftCurlyBrace,
                        Span(self.position - 1, self.position),
                    )
                }
                '}' => {
                    self.position += 1;

                    (
                        Token::RightCurlyBrace,
                        Span(self.position - 1, self.position),
                    )
                }
                '/' => {
                    self.position += 1;

                    (Token::Slash, Span(self.position - 1, self.position))
                }
                '%' => {
                    self.position += 1;

                    (Token::Percent, Span(self.position - 1, self.position))
                }
                '&' => {
                    if let Some('&') = self.peek_second_char() {
                        self.position += 2;

                        (
                            Token::DoubleAmpersand,
                            Span(self.position - 2, self.position),
                        )
                    } else {
                        self.position += 1;

                        return Err(LexError::UnexpectedCharacter {
                            actual: c,
                            position: self.position,
                        });
                    }
                }
                ';' => {
                    self.position += 1;

                    (Token::Semicolon, Span(self.position - 1, self.position))
                }
                '|' => {
                    if let Some('|') = self.peek_second_char() {
                        self.position += 2;

                        (Token::DoublePipe, Span(self.position - 2, self.position))
                    } else {
                        self.position += 1;

                        return Err(LexError::UnexpectedCharacter {
                            actual: c,
                            position: self.position,
                        });
                    }
                }
                '!' => {
                    self.position += 1;

                    (Token::Bang, Span(self.position - 1, self.position))
                }
                ':' => {
                    self.position += 1;

                    (Token::Colon, Span(self.position - 1, self.position))
                }
                _ => {
                    return Err(LexError::UnexpectedCharacter {
                        actual: c,
                        position: self.position,
                    });
                }
            }
        } else {
            (Token::Eof, Span(self.position, self.position))
        };

        Ok((token, span))
    }

    /// Peek at the next token without consuming the source.
    pub fn peek_token(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let token = self.next_token()?;

        self.position -= token.0.len();

        Ok(token)
    }

    /// Progress to the next character.
    fn next_char(&mut self) -> Option<char> {
        if let Some(c) = self.source[self.position..].chars().next() {
            self.position += c.len_utf8();

            Some(c)
        } else {
            None
        }
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

    /// Peek at the second-to-next character without consuming it.
    fn peek_second_char(&self) -> Option<char> {
        self.source[self.position..].chars().nth(1)
    }

    /// Peek the next `n` characters without consuming them.
    fn peek_chars(&self, n: usize) -> &'src str {
        let remaining_source = &self.source[self.position..];

        if remaining_source.len() < n {
            remaining_source
        } else {
            &remaining_source[..n]
        }
    }

    /// Lex an integer or float token.
    fn lex_number(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;
        let mut is_float = false;

        if let Some('-') = self.peek_char() {
            self.next_char();
        }

        while let Some(c) = self.peek_char() {
            if c == '.' {
                if let Some('0'..='9') = self.peek_second_char() {
                    if !is_float {
                        self.next_char();
                    }

                    self.next_char();

                    loop {
                        let peek_char = self.peek_char();

                        if let Some('0'..='9') = peek_char {
                            self.next_char();
                        } else if let Some('e') = peek_char {
                            if let Some('0'..='9') = self.peek_second_char() {
                                self.next_char();
                                self.next_char();
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

            if c == 'x' {
                self.next_char();

                while let Some(c) = self.peek_char() {
                    if c.is_ascii_hexdigit() {
                        self.next_char();
                    } else {
                        break;
                    }
                }

                let text = &self.source[start_pos..self.position];

                return Ok((Token::Byte(text), Span(start_pos, self.position)));
            }

            if c.is_ascii_digit() {
                self.next_char();
            } else {
                break;
            }
        }

        let text = &self.source[start_pos..self.position];

        if is_float {
            Ok((Token::Float(text), Span(start_pos, self.position)))
        } else {
            Ok((Token::Integer(text), Span(start_pos, self.position)))
        }
    }

    /// Lex an identifier token.
    fn lex_alphanumeric(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.next_char();
            } else {
                break;
            }
        }

        let string = &self.source[start_pos..self.position];
        let token = match string {
            "Infinity" => Token::Float("Infinity"),
            "NaN" => Token::Float("NaN"),
            "async" => Token::Async,
            "bool" => Token::Bool,
            "break" => Token::Break,
            "else" => Token::Else,
            "false" => Token::Boolean("false"),
            "float" => Token::FloatKeyword,
            "if" => Token::If,
            "int" => Token::Int,
            "let" => Token::Let,
            "loop" => Token::Loop,
            "map" => Token::Map,
            "mut" => Token::Mut,
            "struct" => Token::Struct,
            "true" => Token::Boolean("true"),
            "while" => Token::While,
            _ => Token::Identifier(string),
        };

        Ok((token, Span(start_pos, self.position)))
    }

    fn lex_string(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        while let Some(c) = self.peek_char() {
            if c == '"' {
                self.next_char();
                break;
            } else {
                self.next_char();
            }
        }

        let text = &self.source[start_pos + 1..self.position - 1];

        Ok((Token::String(text), Span(start_pos, self.position)))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum LexError {
    ExpectedCharacter {
        expected: char,
        actual: char,
        position: usize,
    },
    UnexpectedCharacter {
        actual: char,
        position: usize,
    },
    UnexpectedEndOfFile {
        position: usize,
    },
}

impl LexError {
    pub fn position(&self) -> Span {
        match self {
            Self::ExpectedCharacter { position, .. } => Span(*position, *position),
            Self::UnexpectedCharacter { position, .. } => Span(*position, *position),
            Self::UnexpectedEndOfFile { position } => Span(*position, *position),
        }
    }
}

impl Display for LexError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::ExpectedCharacter {
                expected, actual, ..
            } => write!(f, "Expected character '{expected}', found '{actual}'"),
            Self::UnexpectedCharacter { actual, .. } => {
                write!(f, "Unexpected character '{actual}'")
            }
            Self::UnexpectedEndOfFile { .. } => {
                write!(f, "Unexpected end of file")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character() {
        let input = "'a'";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Character('a'), Span(0, 3)),
                (Token::Eof, Span(3, 3)),
            ])
        );
    }

    #[test]
    fn map_expression() {
        let input = "map { x = \"1\", y = 2, z = 3.0 }";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Map, Span(0, 3)),
                (Token::LeftCurlyBrace, Span(4, 5)),
                (Token::Identifier("x"), Span(6, 7)),
                (Token::Equal, Span(8, 9)),
                (Token::String("1"), Span(10, 13)),
                (Token::Comma, Span(13, 14)),
                (Token::Identifier("y"), Span(15, 16)),
                (Token::Equal, Span(17, 18)),
                (Token::Integer("2"), Span(19, 20)),
                (Token::Comma, Span(20, 21)),
                (Token::Identifier("z"), Span(22, 23)),
                (Token::Equal, Span(24, 25)),
                (Token::Float("3.0"), Span(26, 29)),
                (Token::RightCurlyBrace, Span(30, 31)),
                (Token::Eof, Span(31, 31)),
            ])
        );
    }

    #[test]
    fn let_statement() {
        let input = "let x = 42";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Let, Span(0, 3)),
                (Token::Identifier("x"), Span(4, 5)),
                (Token::Equal, Span(6, 7)),
                (Token::Integer("42"), Span(8, 10)),
                (Token::Eof, Span(10, 10)),
            ])
        );
    }

    #[test]
    fn unit_struct() {
        let input = "struct Foo";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Struct, Span(0, 6)),
                (Token::Identifier("Foo"), Span(7, 10)),
                (Token::Eof, Span(10, 10)),
            ])
        );
    }

    #[test]
    fn tuple_struct() {
        let input = "struct Foo(int, float)";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Struct, Span(0, 6)),
                (Token::Identifier("Foo"), Span(7, 10)),
                (Token::LeftParenthesis, Span(10, 11)),
                (Token::Int, Span(11, 14)),
                (Token::Comma, Span(14, 15)),
                (Token::FloatKeyword, Span(16, 21)),
                (Token::RightParenthesis, Span(21, 22)),
                (Token::Eof, Span(22, 22))
            ])
        );
    }

    #[test]
    fn fields_struct() {
        let input = "struct FooBar { foo: int, bar: float }";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Struct, Span(0, 6)),
                (Token::Identifier("FooBar"), Span(7, 13)),
                (Token::LeftCurlyBrace, Span(14, 15)),
                (Token::Identifier("foo"), Span(16, 19)),
                (Token::Colon, Span(19, 20)),
                (Token::Int, Span(21, 24)),
                (Token::Comma, Span(24, 25)),
                (Token::Identifier("bar"), Span(26, 29)),
                (Token::Colon, Span(29, 30)),
                (Token::FloatKeyword, Span(31, 36)),
                (Token::RightCurlyBrace, Span(37, 38)),
                (Token::Eof, Span(38, 38))
            ])
        );
    }

    #[test]
    fn list_index() {
        let input = "[1, 2, 3][1]";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::LeftSquareBrace, Span(0, 1)),
                (Token::Integer("1"), Span(1, 2)),
                (Token::Comma, Span(2, 3)),
                (Token::Integer("2"), Span(4, 5)),
                (Token::Comma, Span(5, 6)),
                (Token::Integer("3"), Span(7, 8)),
                (Token::RightSquareBrace, Span(8, 9)),
                (Token::LeftSquareBrace, Span(9, 10)),
                (Token::Integer("1"), Span(10, 11)),
                (Token::RightSquareBrace, Span(11, 12)),
                (Token::Eof, Span(12, 12)),
            ])
        )
    }

    #[test]
    fn list() {
        let input = "[1, 2, 3]";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::LeftSquareBrace, Span(0, 1)),
                (Token::Integer("1"), Span(1, 2)),
                (Token::Comma, Span(2, 3)),
                (Token::Integer("2"), Span(4, 5)),
                (Token::Comma, Span(5, 6)),
                (Token::Integer("3"), Span(7, 8)),
                (Token::RightSquareBrace, Span(8, 9)),
                (Token::Eof, Span(9, 9)),
            ])
        )
    }

    #[test]
    fn map_field_access() {
        let input = "{a = 1, b = 2, c = 3}.c";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::LeftCurlyBrace, Span(0, 1)),
                (Token::Identifier("a"), Span(1, 2)),
                (Token::Equal, Span(3, 4)),
                (Token::Integer("1"), Span(5, 6)),
                (Token::Comma, Span(6, 7)),
                (Token::Identifier("b"), Span(8, 9)),
                (Token::Equal, Span(10, 11)),
                (Token::Integer("2"), Span(12, 13)),
                (Token::Comma, Span(13, 14)),
                (Token::Identifier("c"), Span(15, 16)),
                (Token::Equal, Span(17, 18)),
                (Token::Integer("3"), Span(19, 20)),
                (Token::RightCurlyBrace, Span(20, 21)),
                (Token::Dot, Span(21, 22)),
                (Token::Identifier("c"), Span(22, 23)),
                (Token::Eof, Span(23, 23)),
            ])
        )
    }

    #[test]
    fn range() {
        let input = "0..42";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("0"), Span(0, 1)),
                (Token::DoubleDot, Span(1, 3)),
                (Token::Integer("42"), Span(3, 5)),
                (Token::Eof, Span(5, 5))
            ])
        );
    }

    #[test]
    fn negate_expression() {
        let input = "x = -42; -x";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Identifier("x"), Span(0, 1)),
                (Token::Equal, Span(2, 3)),
                (Token::Integer("-42"), Span(4, 7)),
                (Token::Semicolon, Span(7, 8)),
                (Token::Minus, Span(9, 10)),
                (Token::Identifier("x"), Span(10, 11)),
                (Token::Eof, Span(11, 11))
            ])
        );
    }

    #[test]
    fn not_expression() {
        let input = "!true; !false";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Bang, Span(0, 1)),
                (Token::Boolean("true"), Span(1, 5)),
                (Token::Semicolon, Span(5, 6)),
                (Token::Bang, Span(7, 8)),
                (Token::Boolean("false"), Span(8, 13)),
                (Token::Eof, Span(13, 13))
            ])
        );
    }

    #[test]
    fn if_else() {
        let input = "if x < 10 { x + 1 } else { x }";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::If, Span(0, 2)),
                (Token::Identifier("x"), Span(3, 4)),
                (Token::Less, Span(5, 6)),
                (Token::Integer("10"), Span(7, 9)),
                (Token::LeftCurlyBrace, Span(10, 11)),
                (Token::Identifier("x"), Span(12, 13)),
                (Token::Plus, Span(14, 15)),
                (Token::Integer("1"), Span(16, 17)),
                (Token::RightCurlyBrace, Span(18, 19)),
                (Token::Else, Span(20, 24)),
                (Token::LeftCurlyBrace, Span(25, 26)),
                (Token::Identifier("x"), Span(27, 28)),
                (Token::RightCurlyBrace, Span(29, 30)),
                (Token::Eof, Span(30, 30)),
            ])
        )
    }

    #[test]
    fn while_loop() {
        let input = "while x < 10 { x += 1 }";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::While, Span(0, 5)),
                (Token::Identifier("x"), Span(6, 7)),
                (Token::Less, Span(8, 9)),
                (Token::Integer("10"), Span(10, 12)),
                (Token::LeftCurlyBrace, Span(13, 14)),
                (Token::Identifier("x"), Span(15, 16)),
                (Token::PlusEqual, Span(17, 19)),
                (Token::Integer("1"), Span(20, 21)),
                (Token::RightCurlyBrace, Span(22, 23)),
                (Token::Eof, Span(23, 23)),
            ])
        )
    }

    #[test]
    fn add_assign() {
        let input = "x += 42";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Identifier("x"), Span(0, 1)),
                (Token::PlusEqual, Span(2, 4)),
                (Token::Integer("42"), Span(5, 7)),
                (Token::Eof, Span(7, 7)),
            ])
        )
    }

    #[test]
    fn or() {
        let input = "true || false";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Boolean("true"), Span(0, 4)),
                (Token::DoublePipe, Span(5, 7)),
                (Token::Boolean("false"), Span(8, 13)),
                (Token::Eof, Span(13, 13)),
            ])
        )
    }

    #[test]
    fn block() {
        let input = "{ x = 42; y = \"foobar\" }";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::LeftCurlyBrace, Span(0, 1)),
                (Token::Identifier("x"), Span(2, 3)),
                (Token::Equal, Span(4, 5)),
                (Token::Integer("42"), Span(6, 8)),
                (Token::Semicolon, Span(8, 9)),
                (Token::Identifier("y"), Span(10, 11)),
                (Token::Equal, Span(12, 13)),
                (Token::String("foobar"), Span(14, 22)),
                (Token::RightCurlyBrace, Span(23, 24)),
                (Token::Eof, Span(24, 24)),
            ])
        )
    }

    #[test]
    fn equal() {
        let input = "42 == 42";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("42"), Span(0, 2)),
                (Token::DoubleEqual, Span(3, 5)),
                (Token::Integer("42"), Span(6, 8)),
                (Token::Eof, Span(8, 8)),
            ])
        )
    }

    #[test]
    fn modulo() {
        let input = "42 % 2";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("42"), Span(0, 2)),
                (Token::Percent, Span(3, 4)),
                (Token::Integer("2"), Span(5, 6)),
                (Token::Eof, Span(6, 6)),
            ])
        )
    }

    #[test]
    fn divide() {
        let input = "42 / 2";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("42"), Span(0, 2)),
                (Token::Slash, Span(3, 4)),
                (Token::Integer("2"), Span(5, 6)),
                (Token::Eof, Span(6, 6)),
            ])
        )
    }

    #[test]
    fn greater_than() {
        let input = ">";

        assert_eq!(
            lex(input),
            Ok(vec![(Token::Greater, Span(0, 1)), (Token::Eof, Span(1, 1))])
        )
    }

    #[test]
    fn greater_than_or_equal() {
        let input = ">=";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::GreaterEqual, Span(0, 2)),
                (Token::Eof, Span(2, 2))
            ])
        )
    }

    #[test]
    fn less_than() {
        let input = "<";

        assert_eq!(
            lex(input),
            Ok(vec![(Token::Less, Span(0, 1)), (Token::Eof, Span(1, 1))])
        )
    }

    #[test]
    fn less_than_or_equal() {
        let input = "<=";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::LessEqual, Span(0, 2)),
                (Token::Eof, Span(2, 2))
            ])
        )
    }

    #[test]
    fn infinity() {
        let input = "Infinity";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Float("Infinity"), Span(0, 8)),
                (Token::Eof, Span(8, 8)),
            ])
        )
    }

    #[test]
    fn negative_infinity() {
        let input = "-Infinity";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Float("-Infinity"), Span(0, 9)),
                (Token::Eof, Span(9, 9)),
            ])
        )
    }

    #[test]
    fn nan() {
        let input = "NaN";

        assert!(lex(input).is_ok_and(|tokens| tokens[0].0 == Token::Float("NaN")));
    }

    #[test]
    fn complex_float() {
        let input = "42.42e42";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Float("42.42e42"), Span(0, 8)),
                (Token::Eof, Span(8, 8)),
            ])
        )
    }

    #[test]
    fn max_integer() {
        let input = "9223372036854775807";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("9223372036854775807"), Span(0, 19)),
                (Token::Eof, Span(19, 19)),
            ])
        )
    }

    #[test]
    fn min_integer() {
        let input = "-9223372036854775808";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("-9223372036854775808"), Span(0, 20)),
                (Token::Eof, Span(20, 20)),
            ])
        )
    }

    #[test]
    fn subtract_negative_integers() {
        let input = "-42 - -42";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("-42"), Span(0, 3)),
                (Token::Minus, Span(4, 5)),
                (Token::Integer("-42"), Span(6, 9)),
                (Token::Eof, Span(9, 9)),
            ])
        )
    }

    #[test]
    fn negative_integer() {
        let input = "-42";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("-42"), Span(0, 3)),
                (Token::Eof, Span(3, 3))
            ])
        )
    }

    #[test]
    fn read_line() {
        let input = "read_line()";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Identifier("read_line"), Span(0, 9)),
                (Token::LeftParenthesis, Span(9, 10)),
                (Token::RightParenthesis, Span(10, 11)),
                (Token::Eof, Span(11, 11)),
            ])
        )
    }

    #[test]
    fn write_line() {
        let input = "write_line(\"Hello, world!\")";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Identifier("write_line"), Span(0, 10)),
                (Token::LeftParenthesis, Span(10, 11)),
                (Token::String("Hello, world!"), Span(11, 26)),
                (Token::RightParenthesis, Span(26, 27)),
                (Token::Eof, Span(27, 27)),
            ])
        )
    }

    #[test]
    fn string_concatenation() {
        let input = "\"Hello, \" + \"world!\"";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::String("Hello, "), Span(0, 9)),
                (Token::Plus, Span(10, 11)),
                (Token::String("world!"), Span(12, 20)),
                (Token::Eof, Span(20, 20)),
            ])
        )
    }

    #[test]
    fn string() {
        let input = "\"Hello, world!\"";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::String("Hello, world!"), Span(0, 15)),
                (Token::Eof, Span(15, 15)),
            ])
        )
    }

    #[test]
    fn r#true() {
        let input = "true";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Boolean("true"), Span(0, 4)),
                (Token::Eof, Span(4, 4)),
            ])
        )
    }

    #[test]
    fn r#false() {
        let input = "false";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Boolean("false"), Span(0, 5)),
                (Token::Eof, Span(5, 5))
            ])
        )
    }

    #[test]
    fn property_access_function_call() {
        let input = "42.is_even()";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("42"), Span(0, 2)),
                (Token::Dot, Span(2, 3)),
                (Token::Identifier("is_even"), Span(3, 10)),
                (Token::LeftParenthesis, Span(10, 11)),
                (Token::RightParenthesis, Span(11, 12)),
                (Token::Eof, Span(12, 12)),
            ])
        )
    }

    #[test]
    fn empty() {
        let input = "";

        assert_eq!(lex(input), Ok(vec![(Token::Eof, Span(0, 0))]))
    }

    #[test]
    fn reserved_identifier() {
        let input = "length";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Identifier("length"), Span(0, 6)),
                (Token::Eof, Span(6, 6)),
            ])
        )
    }

    #[test]
    fn square_braces() {
        let input = "[]";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::LeftSquareBrace, Span(0, 1)),
                (Token::RightSquareBrace, Span(1, 2)),
                (Token::Eof, Span(2, 2)),
            ])
        )
    }

    #[test]
    fn small_float() {
        let input = "1.23";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Float("1.23"), Span(0, 4)),
                (Token::Eof, Span(4, 4)),
            ])
        )
    }

    #[test]
    #[allow(clippy::excessive_precision)]
    fn big_float() {
        let input = "123456789.123456789";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Float("123456789.123456789"), Span(0, 19)),
                (Token::Eof, Span(19, 19)),
            ])
        )
    }

    #[test]
    fn add() {
        let input = "1 + 2";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("1"), Span(0, 1)),
                (Token::Plus, Span(2, 3)),
                (Token::Integer("2"), Span(4, 5)),
                (Token::Eof, Span(5, 5)),
            ])
        )
    }

    #[test]
    fn multiply() {
        let input = "1 * 2";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("1"), Span(0, 1)),
                (Token::Star, Span(2, 3)),
                (Token::Integer("2"), Span(4, 5)),
                (Token::Eof, Span(5, 5)),
            ])
        )
    }

    #[test]
    fn add_and_multiply() {
        let input = "1 + 2 * 3";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("1"), Span(0, 1)),
                (Token::Plus, Span(2, 3)),
                (Token::Integer("2"), Span(4, 5)),
                (Token::Star, Span(6, 7)),
                (Token::Integer("3"), Span(8, 9)),
                (Token::Eof, Span(9, 9)),
            ])
        );
    }

    #[test]
    fn assignment() {
        let input = "a = 1 + 2 * 3";

        assert_eq!(
            lex(input,),
            Ok(vec![
                (Token::Identifier("a"), Span(0, 1)),
                (Token::Equal, Span(2, 3)),
                (Token::Integer("1"), Span(4, 5)),
                (Token::Plus, Span(6, 7)),
                (Token::Integer("2"), Span(8, 9)),
                (Token::Star, Span(10, 11)),
                (Token::Integer("3"), Span(12, 13)),
                (Token::Eof, Span(13, 13)),
            ])
        );
    }
}
