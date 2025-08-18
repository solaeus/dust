//! Lexing tools and errors
//!
//! This module provides two lexing options:
//! - [`lex`], which lexes the entire input and returns a vector of tokens and their positions
//! - [`Lexer`], which lexes the input a token at a time
use serde::{Deserialize, Serialize};

use crate::{CompileError, DustError, ErrorMessage, Span, Token, dust_error::AnnotatedError};

/// Lexes the input and returns a vector of tokens and their positions.
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
pub fn lex<'a>(source: &'a str) -> Result<Vec<(Token<'a>, Span)>, DustError<'a>> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();

    loop {
        let (token, span) = lexer
            .next_token()
            .map_err(|error| DustError::compile(CompileError::Lex(error), source))?;

        tokens.push((token, span));

        if lexer.is_eof() {
            break;
        }
    }

    Ok(tokens)
}

/// Tool for lexing a single token at a time.
///
/// See the [`lex`] function for an example of how to create and use a Lexer.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Lexer<'src> {
    source: &'src str,
    position: usize,
    is_eof: bool,
}

impl<'src> Lexer<'src> {
    /// Create a new lexer for the given input.
    pub fn new(source: &'src str) -> Self {
        Lexer {
            source,
            position: 0,
            is_eof: false,
        }
    }

    pub fn reset(&mut self, source: &'src str) {
        *self = Lexer::new(source);
    }

    pub fn source(&self) -> &'src str {
        self.source
    }

    pub fn is_eof(&self) -> bool {
        self.is_eof
    }

    pub fn skip_to(&mut self, position: usize) {
        self.position = position;
    }

    /// Produce the next token.
    pub fn next_token(&mut self) -> Result<(Token<'src>, Span), LexError> {
        self.skip_whitespace();

        let (token, span) = if let Some(character) = self.peek_char() {
            let lexer = LexRule::from(&character).lex_action;

            lexer(self)?
        } else {
            self.is_eof = true;

            (Token::Eof, Span(self.position, self.position))
        };

        Ok((token, span))
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
    fn lex_numeric(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;
        let mut is_float = false;
        let peek_char = self.peek_char();

        if let Some('-') = peek_char {
            self.next_char();
        }

        if let Some('0') = peek_char {
            self.next_char();

            if let Some('x') = self.peek_char() {
                self.next_char();

                let mut peek_chars = self.peek_chars(2).chars();

                match (peek_chars.next(), peek_chars.next()) {
                    (Some('0'..='9' | 'A'..='f'), Some('0'..='9' | 'A'..='f')) => {
                        self.next_char();
                        self.next_char();

                        let text = &self.source[start_pos..self.position];

                        return Ok((Token::Byte(text), Span(start_pos, self.position)));
                    }
                    (Some('0'..='9' | 'A'..='f'), erroneous) => {
                        self.next_char();

                        return Err(LexError::ExpectedAsciiHexDigit {
                            actual: erroneous,
                            position: self.position,
                        });
                    }
                    (erroneous, _) => {
                        return Err(LexError::ExpectedAsciiHexDigit {
                            actual: erroneous,
                            position: self.position,
                        });
                    }
                }
            }
        }

        while let Some(c) = self.peek_char() {
            if c == '.' {
                if let Some('0'..='9') = self.peek_second_char() {
                    if !is_float {
                        self.next_char();
                    }

                    is_float = true;

                    self.next_char();

                    while let Some(peek_char) = self.peek_char() {
                        if let '0'..='9' = peek_char {
                            self.next_char();

                            continue;
                        }

                        let peek_second_char = self.peek_second_char();

                        if let ('e' | 'E', Some('0'..='9')) = (peek_char, peek_second_char) {
                            self.next_char();
                            self.next_char();

                            continue;
                        }

                        if let ('e' | 'E', Some('+' | '-')) = (peek_char, peek_second_char) {
                            self.next_char();
                            self.next_char();

                            continue;
                        }

                        break;
                    }
                } else {
                    break;
                }
            }

            if c.is_ascii_digit() || c == '_' {
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
    fn lex_keyword_or_identifier(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.next_char();
            } else if c == ':' && self.peek_second_char() == Some(':') {
                self.next_char();
                self.next_char();
            } else {
                break;
            }
        }

        let string = &self.source[start_pos..self.position];
        let token = match string {
            "Infinity" => Token::Float("Infinity"),
            "NaN" => Token::Float("NaN"),
            "any" => Token::Any,
            "async" => Token::Async,
            "bool" => Token::Bool,
            "break" => Token::Break,
            "cell" => Token::Cell,
            "const" => Token::Const,
            "else" => Token::Else,
            "false" => Token::Boolean("false"),
            "float" => Token::FloatKeyword,
            "fn" => Token::Fn,
            "if" => Token::If,
            "int" => Token::Int,
            "let" => Token::Let,
            "list" => Token::List,
            "loop" => Token::Loop,
            "map" => Token::Map,
            "mod" => Token::Mod,
            "mut" => Token::Mut,
            "return" => Token::Return,
            "str" => Token::Str,
            "struct" => Token::Struct,
            "true" => Token::Boolean("true"),
            "use" => Token::Use,
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

    fn lex_char(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_position = self.position;

        self.next_char();

        let char = self.source[self.position..].chars().next().unwrap();

        self.next_char();

        if self.peek_char() == Some('\'') {
            self.next_char();
        } else {
            return Err(LexError::ExpectedCharacter {
                expected: '\'',
                actual: self.peek_char().unwrap_or('\0'),
                position: self.position,
            });
        }

        let end_position = self.position;

        Ok((Token::Character(char), Span(start_position, end_position)))
    }

    fn lex_plus(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        if let Some('=') = self.peek_char() {
            self.next_char();

            Ok((Token::PlusEqual, Span(start_pos, self.position)))
        } else {
            Ok((Token::Plus, Span(start_pos, self.position)))
        }
    }

    fn lex_minus(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_position = self.position;

        if self
            .peek_second_char()
            .is_some_and(|char| char.is_ascii_digit())
        {
            return self.lex_numeric();
        }

        self.next_char();

        if let Some('=') = self.peek_char() {
            self.next_char();

            return Ok((Token::MinusEqual, Span(start_position, self.position)));
        }

        if let Some('>') = self.peek_char() {
            self.next_char();

            return Ok((Token::ArrowThin, Span(start_position, self.position)));
        }

        if self.peek_chars(8) == "Infinity" {
            self.position += 8;

            return Ok((
                Token::Float("-Infinity"),
                Span(start_position, self.position),
            ));
        }

        Ok((Token::Minus, Span(start_position, self.position)))
    }

    fn lex_star(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        if let Some('=') = self.peek_char() {
            self.next_char();

            Ok((Token::StarEqual, Span(start_pos, self.position)))
        } else {
            Ok((Token::Star, Span(start_pos, self.position)))
        }
    }

    fn lex_slash(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        if let Some('=') = self.peek_char() {
            self.next_char();

            Ok((Token::SlashEqual, Span(start_pos, self.position)))
        } else {
            Ok((Token::Slash, Span(start_pos, self.position)))
        }
    }

    fn lex_percent(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        if let Some('=') = self.peek_char() {
            self.next_char();

            Ok((Token::PercentEqual, Span(start_pos, self.position)))
        } else {
            Ok((Token::Percent, Span(start_pos, self.position)))
        }
    }

    fn lex_unexpected(&mut self) -> Result<(Token<'src>, Span), LexError> {
        Err(LexError::UnexpectedCharacter {
            actual: self.peek_char().unwrap_or('\0'),
            position: self.position,
        })
    }

    fn lex_exclamation_mark(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        if let Some('=') = self.peek_char() {
            self.next_char();

            Ok((Token::BangEqual, Span(start_pos, self.position)))
        } else {
            Ok((Token::Bang, Span(start_pos, self.position)))
        }
    }

    fn lex_equal(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        if let Some('=') = self.peek_char() {
            self.next_char();

            Ok((Token::DoubleEqual, Span(start_pos, self.position)))
        } else {
            Ok((Token::Equal, Span(start_pos, self.position)))
        }
    }

    fn lex_less_than(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        if let Some('=') = self.peek_char() {
            self.next_char();

            Ok((Token::LessEqual, Span(start_pos, self.position)))
        } else {
            Ok((Token::Less, Span(start_pos, self.position)))
        }
    }

    fn lex_greater_than(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        if let Some('=') = self.peek_char() {
            self.next_char();

            Ok((Token::GreaterEqual, Span(start_pos, self.position)))
        } else {
            Ok((Token::Greater, Span(start_pos, self.position)))
        }
    }

    fn lex_ampersand(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        let peek_char = self.peek_char();

        if let Some('&') = peek_char {
            self.next_char();

            Ok((Token::DoubleAmpersand, Span(start_pos, self.position)))
        } else if peek_char.is_none() {
            Err(LexError::UnexpectedEndOfFile {
                position: self.position,
            })
        } else {
            Err(LexError::ExpectedCharacter {
                expected: '&',
                actual: self.peek_char().unwrap(),
                position: self.position,
            })
        }
    }

    fn lex_pipe(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        let peek_char = self.peek_char();

        if let Some('|') = peek_char {
            self.next_char();

            Ok((Token::DoublePipe, Span(start_pos, self.position)))
        } else if peek_char.is_none() {
            Err(LexError::UnexpectedEndOfFile {
                position: self.position,
            })
        } else {
            Err(LexError::ExpectedCharacter {
                expected: '&',
                actual: self.peek_char().unwrap(),
                position: self.position,
            })
        }
    }

    fn lex_left_parenthesis(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        Ok((Token::LeftParenthesis, Span(start_pos, self.position)))
    }

    fn lex_right_parenthesis(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        Ok((Token::RightParenthesis, Span(start_pos, self.position)))
    }

    fn lex_left_bracket(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        Ok((Token::LeftBracket, Span(start_pos, self.position)))
    }

    fn lex_right_bracket(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        Ok((Token::RightBracket, Span(start_pos, self.position)))
    }

    fn lex_left_brace(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        Ok((Token::LeftBrace, Span(start_pos, self.position)))
    }

    fn lex_right_brace(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        Ok((Token::RightBrace, Span(start_pos, self.position)))
    }

    fn lex_semicolon(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        Ok((Token::Semicolon, Span(start_pos, self.position)))
    }

    fn lex_colon(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        Ok((Token::Colon, Span(start_pos, self.position)))
    }

    fn lex_comma(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        Ok((Token::Comma, Span(start_pos, self.position)))
    }

    fn lex_dot(&mut self) -> Result<(Token<'src>, Span), LexError> {
        let start_pos = self.position;

        self.next_char();

        if let Some('.') = self.peek_char() {
            self.next_char();

            Ok((Token::DoubleDot, Span(start_pos, self.position)))
        } else {
            Ok((Token::Dot, Span(start_pos, self.position)))
        }
    }
}

type LexAction<'src> = fn(&mut Lexer<'src>) -> Result<(Token<'src>, Span), LexError>;

pub struct LexRule<'src> {
    lex_action: LexAction<'src>,
}

impl From<&char> for LexRule<'_> {
    fn from(char: &char) -> Self {
        match char {
            '0'..='9' => LexRule {
                lex_action: Lexer::lex_numeric,
            },
            char if char.is_alphabetic() => LexRule {
                lex_action: Lexer::lex_keyword_or_identifier,
            },
            '_' => LexRule {
                lex_action: Lexer::lex_keyword_or_identifier,
            },
            '"' => LexRule {
                lex_action: Lexer::lex_string,
            },
            '\'' => LexRule {
                lex_action: Lexer::lex_char,
            },
            '+' => LexRule {
                lex_action: Lexer::lex_plus,
            },
            '-' => LexRule {
                lex_action: Lexer::lex_minus,
            },
            '*' => LexRule {
                lex_action: Lexer::lex_star,
            },
            '/' => LexRule {
                lex_action: Lexer::lex_slash,
            },
            '%' => LexRule {
                lex_action: Lexer::lex_percent,
            },
            '!' => LexRule {
                lex_action: Lexer::lex_exclamation_mark,
            },
            '=' => LexRule {
                lex_action: Lexer::lex_equal,
            },
            '<' => LexRule {
                lex_action: Lexer::lex_less_than,
            },
            '>' => LexRule {
                lex_action: Lexer::lex_greater_than,
            },
            '&' => LexRule {
                lex_action: Lexer::lex_ampersand,
            },
            '|' => LexRule {
                lex_action: Lexer::lex_pipe,
            },
            '(' => LexRule {
                lex_action: Lexer::lex_left_parenthesis,
            },
            ')' => LexRule {
                lex_action: Lexer::lex_right_parenthesis,
            },
            '[' => LexRule {
                lex_action: Lexer::lex_left_bracket,
            },
            ']' => LexRule {
                lex_action: Lexer::lex_right_bracket,
            },
            '{' => LexRule {
                lex_action: Lexer::lex_left_brace,
            },
            '}' => LexRule {
                lex_action: Lexer::lex_right_brace,
            },
            ';' => LexRule {
                lex_action: Lexer::lex_semicolon,
            },
            ':' => LexRule {
                lex_action: Lexer::lex_colon,
            },
            ',' => LexRule {
                lex_action: Lexer::lex_comma,
            },
            '.' => LexRule {
                lex_action: Lexer::lex_dot,
            },
            _ => LexRule {
                lex_action: Lexer::lex_unexpected,
            },
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum LexError {
    ExpectedAsciiHexDigit {
        actual: Option<char>,
        position: usize,
    },
    ExpectedCharacter {
        expected: char,
        actual: char,
        position: usize,
    },
    ExpectedCharacterMultiple {
        expected: &'static [char],
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

impl AnnotatedError for LexError {
    fn annotated_error(&self) -> ErrorMessage {
        let title = "Lexing Error";
        let (description, detail_snippets, help_snippet) = match self {
            LexError::ExpectedAsciiHexDigit { actual, position } => (
                "Expected an ASCII hex digit",
                vec![(
                    format!("Expected an ASCII hex digit, found '{actual:?}'"),
                    Span(*position, *position + 1),
                )],
                None,
            ),
            LexError::ExpectedCharacter {
                expected,
                actual,
                position,
            } => (
                "Expected a character",
                vec![(
                    format!("Expected '{expected}', found '{actual}'"),
                    Span(*position, *position + 1),
                )],
                None,
            ),
            LexError::ExpectedCharacterMultiple {
                expected,
                actual,
                position,
            } => (
                "Expected a character",
                vec![(
                    format!("Expected one of '{expected:?}', found '{actual}'"),
                    Span(*position, *position + 1),
                )],
                None,
            ),
            LexError::UnexpectedCharacter { actual, position } => (
                "Unexpected character",
                vec![(
                    format!("Unexpected character '{actual}'"),
                    Span(*position, *position + 1),
                )],
                None,
            ),
            LexError::UnexpectedEndOfFile { position } => (
                "Unexpected end of file",
                vec![(
                    "Unexpected end of file".to_string(),
                    Span(*position, *position),
                )],
                None,
            ),
        };

        ErrorMessage {
            title,
            description,
            detail_snippets,
            help_snippet,
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
                (Token::LeftBrace, Span(4, 5)),
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
                (Token::RightBrace, Span(30, 31)),
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
                (Token::LeftBrace, Span(14, 15)),
                (Token::Identifier("foo"), Span(16, 19)),
                (Token::Colon, Span(19, 20)),
                (Token::Int, Span(21, 24)),
                (Token::Comma, Span(24, 25)),
                (Token::Identifier("bar"), Span(26, 29)),
                (Token::Colon, Span(29, 30)),
                (Token::FloatKeyword, Span(31, 36)),
                (Token::RightBrace, Span(37, 38)),
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
                (Token::LeftBracket, Span(0, 1)),
                (Token::Integer("1"), Span(1, 2)),
                (Token::Comma, Span(2, 3)),
                (Token::Integer("2"), Span(4, 5)),
                (Token::Comma, Span(5, 6)),
                (Token::Integer("3"), Span(7, 8)),
                (Token::RightBracket, Span(8, 9)),
                (Token::LeftBracket, Span(9, 10)),
                (Token::Integer("1"), Span(10, 11)),
                (Token::RightBracket, Span(11, 12)),
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
                (Token::LeftBracket, Span(0, 1)),
                (Token::Integer("1"), Span(1, 2)),
                (Token::Comma, Span(2, 3)),
                (Token::Integer("2"), Span(4, 5)),
                (Token::Comma, Span(5, 6)),
                (Token::Integer("3"), Span(7, 8)),
                (Token::RightBracket, Span(8, 9)),
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
                (Token::LeftBrace, Span(0, 1)),
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
                (Token::RightBrace, Span(20, 21)),
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
                (Token::LeftBrace, Span(10, 11)),
                (Token::Identifier("x"), Span(12, 13)),
                (Token::Plus, Span(14, 15)),
                (Token::Integer("1"), Span(16, 17)),
                (Token::RightBrace, Span(18, 19)),
                (Token::Else, Span(20, 24)),
                (Token::LeftBrace, Span(25, 26)),
                (Token::Identifier("x"), Span(27, 28)),
                (Token::RightBrace, Span(29, 30)),
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
                (Token::LeftBrace, Span(13, 14)),
                (Token::Identifier("x"), Span(15, 16)),
                (Token::PlusEqual, Span(17, 19)),
                (Token::Integer("1"), Span(20, 21)),
                (Token::RightBrace, Span(22, 23)),
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
                (Token::LeftBrace, Span(0, 1)),
                (Token::Identifier("x"), Span(2, 3)),
                (Token::Equal, Span(4, 5)),
                (Token::Integer("42"), Span(6, 8)),
                (Token::Semicolon, Span(8, 9)),
                (Token::Identifier("y"), Span(10, 11)),
                (Token::Equal, Span(12, 13)),
                (Token::String("foobar"), Span(14, 22)),
                (Token::RightBrace, Span(23, 24)),
                (Token::Eof, Span(24, 24)),
            ])
        )
    }

    #[test]
    fn not_equal() {
        let input = "42 != 42";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer("42"), Span(0, 2)),
                (Token::BangEqual, Span(3, 5)),
                (Token::Integer("42"), Span(6, 8)),
                (Token::Eof, Span(8, 8)),
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
                (Token::LeftBracket, Span(0, 1)),
                (Token::RightBracket, Span(1, 2)),
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
    fn float_with_exponent() {
        let input = "1.23e4";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Float("1.23e4"), Span(0, 6)),
                (Token::Eof, Span(6, 6)),
            ])
        )
    }

    #[test]
    fn float_with_negative_exponent() {
        let input = "1.23e-4";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Float("1.23e-4"), Span(0, 7)),
                (Token::Eof, Span(7, 7)),
            ])
        )
    }

    #[test]
    fn float_infinity_and_nan() {
        let input = "Infinity -Infinity NaN";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Float("Infinity"), Span(0, 8)),
                (Token::Float("-Infinity"), Span(9, 18)),
                (Token::Float("NaN"), Span(19, 22)),
                (Token::Eof, Span(22, 22)),
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
