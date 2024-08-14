//! Lexing tools.
//!
//! This module provides two lexing options:
//! - [`lex`], which lexes the entire input and returns a vector of tokens and their positions
//! - [`Lexer`], which lexes the input a token at a time
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
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
///         (Token::Integer("1"), (4, 5)),
///         (Token::Plus, (6, 7)),
///         (Token::Integer("2"), (8, 9)),
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
///         (Token::Integer("1"), (4, 5)),
///         (Token::Plus, (6, 7)),
///         (Token::Integer("2"), (8, 9)),
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
                    let second_char = self.peek_second_char(source);

                    if let Some('=') = second_char {
                        self.position += 2;

                        (Token::MinusEqual, (self.position - 2, self.position))
                    } else if let Some('0'..='9') = second_char {
                        self.lex_number(source)?
                    } else if "-Infinity" == self.peek_chars(source, 9) {
                        self.position += 9;

                        (
                            Token::Float("-Infinity"),
                            (self.position - 9, self.position),
                        )
                    } else {
                        self.position += 1;

                        (Token::Minus, (self.position - 1, self.position))
                    }
                }
                'a'..='z' | 'A'..='Z' => self.lex_alphanumeric(source)?,
                '"' => self.lex_string('"', source)?,
                '\'' => self.lex_string('\'', source)?,
                '+' => {
                    if let Some('=') = self.peek_second_char(source) {
                        self.position += 2;

                        (Token::PlusEqual, (self.position - 2, self.position))
                    } else {
                        self.position += 1;

                        (Token::Plus, (self.position - 1, self.position))
                    }
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
                    if let Some('=') = self.peek_second_char(source) {
                        self.position += 2;

                        (Token::DoubleEqual, (self.position - 2, self.position))
                    } else {
                        self.position += 1;

                        (Token::Equal, (self.position - 1, self.position))
                    }
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
                    if let Some('.') = self.peek_second_char(source) {
                        self.position += 2;

                        (Token::DoubleDot, (self.position - 2, self.position))
                    } else {
                        self.position += 1;

                        (Token::Dot, (self.position - 1, self.position))
                    }
                }
                '>' => {
                    if let Some('=') = self.peek_second_char(source) {
                        self.position += 2;

                        (Token::GreaterEqual, (self.position - 2, self.position))
                    } else {
                        self.position += 1;

                        (Token::Greater, (self.position - 1, self.position))
                    }
                }
                '<' => {
                    if let Some('=') = self.peek_second_char(source) {
                        self.position += 2;

                        (Token::LessEqual, (self.position - 2, self.position))
                    } else {
                        self.position += 1;

                        (Token::Less, (self.position - 1, self.position))
                    }
                }
                '{' => {
                    self.position += 1;

                    (Token::LeftCurlyBrace, (self.position - 1, self.position))
                }
                '}' => {
                    self.position += 1;

                    (Token::RightCurlyBrace, (self.position - 1, self.position))
                }
                '/' => {
                    self.position += 1;

                    (Token::Slash, (self.position - 1, self.position))
                }
                '%' => {
                    self.position += 1;

                    (Token::Percent, (self.position - 1, self.position))
                }
                '&' => {
                    if let Some('&') = self.peek_second_char(source) {
                        self.position += 2;

                        (Token::DoubleAmpersand, (self.position - 2, self.position))
                    } else {
                        self.position += 1;

                        return Err(LexError::UnexpectedCharacter {
                            character: c,
                            position: self.position,
                        });
                    }
                }
                ';' => {
                    self.position += 1;

                    (Token::Semicolon, (self.position - 1, self.position))
                }
                '|' => {
                    if let Some('|') = self.peek_second_char(source) {
                        self.position += 2;

                        (Token::DoublePipe, (self.position - 2, self.position))
                    } else {
                        self.position += 1;

                        return Err(LexError::UnexpectedCharacter {
                            character: c,
                            position: self.position,
                        });
                    }
                }
                '!' => {
                    self.position += 1;

                    (Token::Bang, (self.position - 1, self.position))
                }
                ':' => {
                    self.position += 1;

                    (Token::Colon, (self.position - 1, self.position))
                }
                _ => {
                    self.position += 1;

                    return Err(LexError::UnexpectedCharacter {
                        character: c,
                        position: self.position,
                    });
                }
            }
        } else {
            (Token::Eof, (self.position, self.position))
        };

        Ok((token, span))
    }

    /// Peek at the next token without consuming the source.
    pub fn peek_token<'src>(&mut self, source: &'src str) -> Result<(Token<'src>, Span), LexError> {
        let token = self.next_token(source)?;

        self.position -= token.0.as_str().len();

        Ok(token)
    }

    /// Progress to the next character.
    fn next_char(&mut self, source: &str) -> Option<char> {
        if let Some(c) = source[self.position..].chars().next() {
            self.position += c.len_utf8();

            Some(c)
        } else {
            None
        }
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

        let text = &source[start_pos..self.position];

        if is_float {
            Ok((Token::Float(text), (start_pos, self.position)))
        } else {
            Ok((Token::Integer(text), (start_pos, self.position)))
        }
    }

    /// Lex an identifier token.
    fn lex_alphanumeric<'src>(
        &mut self,
        source: &'src str,
    ) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        while let Some(c) = self.peek_char(source) {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.next_char(source);
            } else {
                break;
            }
        }

        let string = &source[start_pos..self.position];
        let token = match string {
            "Infinity" => Token::Float("Infinity"),
            "NaN" => Token::Float("NaN"),
            "async" => Token::Async,
            "bool" => Token::Bool,
            "else" => Token::Else,
            "false" => Token::Boolean("false"),
            "float" => Token::FloatKeyword,
            "if" => Token::If,
            "int" => Token::Int,
            "is_even" => Token::IsEven,
            "is_odd" => Token::IsOdd,
            "length" => Token::Length,
            "read_line" => Token::ReadLine,
            "struct" => Token::Struct,
            "to_string" => Token::ToString,
            "true" => Token::Boolean("true"),
            "while" => Token::While,
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
    UnexpectedCharacter { character: char, position: usize },
}

impl LexError {
    pub fn position(&self) -> Span {
        match self {
            Self::UnexpectedCharacter { position, .. } => (*position, *position),
        }
    }
}

impl Error for LexError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnexpectedCharacter { .. } => None,
        }
    }
}

impl Display for LexError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedCharacter { character, .. } => {
                write!(f, "Unexpected character: '{}'", character)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_keywords() {
        let input = "async bool else false float if int is_even is_odd length read_line struct to_string true while write_line";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Async, (0, 5)),
                (Token::Bool, (6, 10)),
                (Token::Else, (11, 15)),
                (Token::Boolean("false"), (16, 21)),
                (Token::FloatKeyword, (22, 27)),
                (Token::If, (28, 30)),
                (Token::Int, (31, 34)),
                (Token::IsEven, (35, 42)),
                (Token::IsOdd, (43, 49)),
                (Token::Length, (50, 56)),
                (Token::ReadLine, (57, 66)),
                (Token::Struct, (67, 73)),
                (Token::ToString, (74, 83)),
                (Token::Boolean("true"), (84, 88)),
                (Token::While, (89, 94)),
                (Token::WriteLine, (95, 105)),
                (Token::Eof, (105, 105)),
            ])
        );
    }

    #[test]
    fn unit_struct() {
        let input = "struct Foo";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Struct, (0, 6)),
                (Token::Identifier("Foo"), (7, 10)),
                (Token::Eof, (10, 10)),
            ])
        );
    }

    #[test]
    fn tuple_struct() {
        let input = "struct Foo(int, float)";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Struct, (0, 6)),
                (Token::Identifier("Foo"), (7, 10)),
                (Token::LeftParenthesis, (10, 11)),
                (Token::Int, (11, 14)),
                (Token::Comma, (14, 15)),
                (Token::FloatKeyword, (16, 21)),
                (Token::RightParenthesis, (21, 22)),
                (Token::Eof, (22, 22))
            ])
        );
    }

    #[test]
    fn fields_struct() {
        let input = "struct FooBar { foo: int, bar: float }";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Struct, (0, 6)),
                (Token::Identifier("FooBar"), (7, 13)),
                (Token::LeftCurlyBrace, (14, 15)),
                (Token::Identifier("foo"), (16, 19)),
                (Token::Colon, (19, 20)),
                (Token::Int, (21, 24)),
                (Token::Comma, (24, 25)),
                (Token::Identifier("bar"), (26, 29)),
                (Token::Colon, (29, 30)),
                (Token::FloatKeyword, (31, 36)),
                (Token::RightCurlyBrace, (37, 38)),
                (Token::Eof, (38, 38))
            ])
        );
    }

    #[test]
    fn list_index() {
        let input = "[1, 2, 3][1]";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::LeftSquareBrace, (0, 1)),
                (Token::Integer("1"), (1, 2)),
                (Token::Comma, (2, 3)),
                (Token::Integer("2"), (4, 5)),
                (Token::Comma, (5, 6)),
                (Token::Integer("3"), (7, 8)),
                (Token::RightSquareBrace, (8, 9)),
                (Token::LeftSquareBrace, (9, 10)),
                (Token::Integer("1"), (10, 11)),
                (Token::RightSquareBrace, (11, 12)),
                (Token::Eof, (12, 12)),
            ])
        )
    }

    #[test]
    fn list() {
        let input = "[1, 2, 3]";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::LeftSquareBrace, (0, 1)),
                (Token::Integer("1"), (1, 2)),
                (Token::Comma, (2, 3)),
                (Token::Integer("2"), (4, 5)),
                (Token::Comma, (5, 6)),
                (Token::Integer("3"), (7, 8)),
                (Token::RightSquareBrace, (8, 9)),
                (Token::Eof, (9, 9)),
            ])
        )
    }

    #[test]
    fn map_field_access() {
        let input = "{a = 1, b = 2, c = 3}.c";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::LeftCurlyBrace, (0, 1)),
                (Token::Identifier("a"), (1, 2)),
                (Token::Equal, (3, 4)),
                (Token::Integer("1"), (5, 6)),
                (Token::Comma, (6, 7)),
                (Token::Identifier("b"), (8, 9)),
                (Token::Equal, (10, 11)),
                (Token::Integer("2"), (12, 13)),
                (Token::Comma, (13, 14)),
                (Token::Identifier("c"), (15, 16)),
                (Token::Equal, (17, 18)),
                (Token::Integer("3"), (19, 20)),
                (Token::RightCurlyBrace, (20, 21)),
                (Token::Dot, (21, 22)),
                (Token::Identifier("c"), (22, 23)),
                (Token::Eof, (23, 23)),
            ])
        )
    }
    #[test]
    fn range() {
        let input = "0..42";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("0"), (0, 1)),
                (Token::DoubleDot, (1, 3)),
                (Token::Integer("42"), (3, 5)),
                (Token::Eof, (5, 5))
            ])
        );
    }

    #[test]
    fn negate_expression() {
        let input = "x = -42; -x";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Identifier("x"), (0, 1)),
                (Token::Equal, (2, 3)),
                (Token::Integer("-42"), (4, 7)),
                (Token::Semicolon, (7, 8)),
                (Token::Minus, (9, 10)),
                (Token::Identifier("x"), (10, 11)),
                (Token::Eof, (11, 11))
            ])
        );
    }

    #[test]
    fn not_expression() {
        let input = "!true; !false";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Bang, (0, 1)),
                (Token::Boolean("true"), (1, 5)),
                (Token::Semicolon, (5, 6)),
                (Token::Bang, (7, 8)),
                (Token::Boolean("false"), (8, 13)),
                (Token::Eof, (13, 13))
            ])
        );
    }

    #[test]
    fn if_else() {
        let input = "if x < 10 { x + 1 } else { x }";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::If, (0, 2)),
                (Token::Identifier("x"), (3, 4)),
                (Token::Less, (5, 6)),
                (Token::Integer("10"), (7, 9)),
                (Token::LeftCurlyBrace, (10, 11)),
                (Token::Identifier("x"), (12, 13)),
                (Token::Plus, (14, 15)),
                (Token::Integer("1"), (16, 17)),
                (Token::RightCurlyBrace, (18, 19)),
                (Token::Else, (20, 24)),
                (Token::LeftCurlyBrace, (25, 26)),
                (Token::Identifier("x"), (27, 28)),
                (Token::RightCurlyBrace, (29, 30)),
                (Token::Eof, (30, 30)),
            ])
        )
    }

    #[test]
    fn while_loop() {
        let input = "while x < 10 { x += 1 }";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::While, (0, 5)),
                (Token::Identifier("x"), (6, 7)),
                (Token::Less, (8, 9)),
                (Token::Integer("10"), (10, 12)),
                (Token::LeftCurlyBrace, (13, 14)),
                (Token::Identifier("x"), (15, 16)),
                (Token::PlusEqual, (17, 19)),
                (Token::Integer("1"), (20, 21)),
                (Token::RightCurlyBrace, (22, 23)),
                (Token::Eof, (23, 23)),
            ])
        )
    }

    #[test]
    fn add_assign() {
        let input = "x += 42";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Identifier("x"), (0, 1)),
                (Token::PlusEqual, (2, 4)),
                (Token::Integer("42"), (5, 7)),
                (Token::Eof, (7, 7)),
            ])
        )
    }

    #[test]
    fn or() {
        let input = "true || false";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Boolean("true"), (0, 4)),
                (Token::DoublePipe, (5, 7)),
                (Token::Boolean("false"), (8, 13)),
                (Token::Eof, (13, 13)),
            ])
        )
    }

    #[test]
    fn block() {
        let input = "{ x = 42; y = 'foobar' }";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::LeftCurlyBrace, (0, 1)),
                (Token::Identifier("x"), (2, 3)),
                (Token::Equal, (4, 5)),
                (Token::Integer("42"), (6, 8)),
                (Token::Semicolon, (8, 9)),
                (Token::Identifier("y"), (10, 11)),
                (Token::Equal, (12, 13)),
                (Token::String("foobar"), (14, 22)),
                (Token::RightCurlyBrace, (23, 24)),
                (Token::Eof, (24, 24)),
            ])
        )
    }

    #[test]
    fn equal() {
        let input = "42 == 42";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("42"), (0, 2)),
                (Token::DoubleEqual, (3, 5)),
                (Token::Integer("42"), (6, 8)),
                (Token::Eof, (8, 8)),
            ])
        )
    }

    #[test]
    fn modulo() {
        let input = "42 % 2";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("42"), (0, 2)),
                (Token::Percent, (3, 4)),
                (Token::Integer("2"), (5, 6)),
                (Token::Eof, (6, 6)),
            ])
        )
    }

    #[test]
    fn divide() {
        let input = "42 / 2";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("42"), (0, 2)),
                (Token::Slash, (3, 4)),
                (Token::Integer("2"), (5, 6)),
                (Token::Eof, (6, 6)),
            ])
        )
    }

    #[test]
    fn map() {
        let input = "{ x = 42, y = 'foobar' }";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::LeftCurlyBrace, (0, 1)),
                (Token::Identifier("x"), (2, 3)),
                (Token::Equal, (4, 5)),
                (Token::Integer("42"), (6, 8)),
                (Token::Comma, (8, 9)),
                (Token::Identifier("y"), (10, 11)),
                (Token::Equal, (12, 13)),
                (Token::String("foobar"), (14, 22)),
                (Token::RightCurlyBrace, (23, 24)),
                (Token::Eof, (24, 24)),
            ])
        )
    }

    #[test]
    fn greater_than() {
        let input = ">";

        assert_eq!(
            lex(input),
            Ok(vec![(Token::Greater, (0, 1)), (Token::Eof, (1, 1))])
        )
    }

    #[test]
    fn greater_than_or_equal() {
        let input = ">=";

        assert_eq!(
            lex(input),
            Ok(vec![(Token::GreaterEqual, (0, 2)), (Token::Eof, (2, 2))])
        )
    }

    #[test]
    fn less_than() {
        let input = "<";

        assert_eq!(
            lex(input),
            Ok(vec![(Token::Less, (0, 1)), (Token::Eof, (1, 1))])
        )
    }

    #[test]
    fn less_than_or_equal() {
        let input = "<=";

        assert_eq!(
            lex(input),
            Ok(vec![(Token::LessEqual, (0, 2)), (Token::Eof, (2, 2))])
        )
    }

    #[test]
    fn infinity() {
        let input = "Infinity";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Float("Infinity"), (0, 8)),
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
                (Token::Float("-Infinity"), (0, 9)),
                (Token::Eof, (9, 9)),
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
                (Token::Float("42.42e42"), (0, 8)),
                (Token::Eof, (8, 8)),
            ])
        )
    }

    #[test]
    fn max_integer() {
        let input = "9223372036854775807";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("9223372036854775807"), (0, 19)),
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
                (Token::Integer("-9223372036854775808"), (0, 20)),
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
                (Token::Integer("-42"), (0, 3)),
                (Token::Minus, (4, 5)),
                (Token::Integer("-42"), (6, 9)),
                (Token::Eof, (9, 9)),
            ])
        )
    }

    #[test]
    fn negative_integer() {
        let input = "-42";

        assert_eq!(
            lex(input),
            Ok(vec![(Token::Integer("-42"), (0, 3)), (Token::Eof, (3, 3))])
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
            Ok(vec![(Token::Boolean("true"), (0, 4)), (Token::Eof, (4, 4)),])
        )
    }

    #[test]
    fn r#false() {
        let input = "false";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Boolean("false"), (0, 5)),
                (Token::Eof, (5, 5))
            ])
        )
    }

    #[test]
    fn property_access_function_call() {
        let input = "42.is_even()";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("42"), (0, 2)),
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
            Ok(vec![(Token::Float("1.23"), (0, 4)), (Token::Eof, (4, 4)),])
        )
    }

    #[test]
    #[allow(clippy::excessive_precision)]
    fn big_float() {
        let input = "123456789.123456789";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Float("123456789.123456789"), (0, 19)),
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
                (Token::Integer("1"), (0, 1)),
                (Token::Plus, (2, 3)),
                (Token::Integer("2"), (4, 5)),
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
                (Token::Integer("1"), (0, 1)),
                (Token::Star, (2, 3)),
                (Token::Integer("2"), (4, 5)),
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
                (Token::Integer("1"), (0, 1)),
                (Token::Plus, (2, 3)),
                (Token::Integer("2"), (4, 5)),
                (Token::Star, (6, 7)),
                (Token::Integer("3"), (8, 9)),
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
                (Token::Integer("1"), (4, 5)),
                (Token::Plus, (6, 7)),
                (Token::Integer("2"), (8, 9)),
                (Token::Star, (10, 11)),
                (Token::Integer("3"), (12, 13)),
                (Token::Eof, (13, 13)),
            ])
        );
    }
}
