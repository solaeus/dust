use tracing::{info, trace};

use crate::{Span, TokenKind, token::Token};

pub fn tokenize(source: &[u8]) -> Vec<Token> {
    let mut lexer = Lexer::new();
    let mut tokens = Vec::new();

    lexer.initialize(source);

    while let Some(token) = lexer.next_token() {
        tokens.push(token);
    }

    tokens
}

#[derive(Debug, Default)]
pub struct Lexer<'src> {
    source: &'src [u8],
    current_index: usize,
}

impl<'src> Lexer<'src> {
    pub fn new() -> Self {
        Lexer {
            source: &[],
            current_index: 0,
        }
    }

    #[inline]
    pub fn initialize(&mut self, source: &'src [u8]) {
        self.source = source;
        self.current_index = 0;
    }

    #[inline]
    pub fn source(&self) -> &'src [u8] {
        self.source
    }

    #[inline]
    pub fn is_at_eof(&self) -> bool {
        self.current_index >= self.source.len()
    }

    /// Produce the next token.
    #[inline]
    pub fn next_token(&mut self) -> Option<Token> {
        if self.is_at_eof() {
            return None;
        }

        let next = match self.current_byte() {
            b'0'..=b'9' => self.lex_numeric(),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.lex_keyword_or_identifier(),
            b'"' => self.lex_string(),
            b'\'' => self.lex_character(),
            b'+' => self.lex_plus(),
            b'-' => self.lex_minus(),
            b'*' => self.lex_asterisk(),
            b'/' => self.lex_slash(),
            b'%' => self.lex_percent(),
            b'!' => self.lex_exclamation_mark(),
            b'=' => self.lex_equal(),
            b'<' => self.lex_less_than(),
            b'>' => self.lex_greater_than(),
            b'&' => self.lex_ampersand(),
            b'|' => self.lex_pipe(),
            b'.' => self.lex_dot(),
            b':' => self.lex_colon(),
            b'(' => self.lex_byte(TokenKind::LeftParenthesis),
            b')' => self.lex_byte(TokenKind::RightParenthesis),
            b'[' => self.lex_byte(TokenKind::LeftSquareBracket),
            b']' => self.lex_byte(TokenKind::RightSquareBracket),
            b'{' => self.lex_byte(TokenKind::LeftCurlyBrace),
            b'}' => self.lex_byte(TokenKind::RightCurlyBrace),
            b';' => self.lex_byte(TokenKind::Semicolon),
            b',' => self.lex_byte(TokenKind::Comma),
            b' ' => self.lex_byte(TokenKind::Space),
            b'\t' => self.lex_byte(TokenKind::Tab),
            b'\r' => self.lex_carriage_return(),
            b'\n' => self.lex_byte(TokenKind::Newline),
            _ => {
                self.advance();

                self.lex_byte(TokenKind::Unknown)
            }
        };

        Some(next)
    }

    /// Emit a token for a one-byte character.
    #[inline]
    fn lex_byte(&mut self, kind: TokenKind) -> Token {
        info!("Emitting token: {}", kind);

        let start = self.current_index;

        self.advance();

        Token {
            kind,
            span: Span::new(start, self.current_index),
        }
    }

    /// Progress to the next character.
    #[inline]
    fn advance(&mut self) {
        let next_position = self.current_index + 1;

        if next_position <= self.source.len() {
            self.current_index = next_position;
        }
    }

    /// Peek at the next character without consuming it.
    #[inline]
    fn current_byte(&self) -> u8 {
        if self.current_index < self.source.len() {
            self.source[self.current_index]
        } else {
            0
        }
    }

    /// Peek at the second-to-next character without consuming it.
    #[inline]
    fn next_byte(&self) -> Option<u8> {
        let next_position = self.current_index + 1;

        if next_position < self.source.len() {
            Some(self.source[next_position])
        } else {
            None
        }
    }

    /// Peek the next `n` characters without consuming them.
    #[inline]
    fn peek_bytes(&self, n: usize) -> &'src [u8] {
        let end_position = (self.current_index + n).min(self.source.len());

        &self.source[self.current_index..end_position]
    }

    #[inline]
    fn lex_numeric(&mut self) -> Token {
        trace!("Lexing numeric");

        let start = self.current_index;
        let mut is_float = false;
        let byte = self.current_byte();

        if byte == b'-' {
            self.advance();
        }

        if byte == b'0' {
            self.advance();

            if self.current_byte() == b'x' {
                self.advance();

                for byte in self.peek_bytes(2) {
                    if byte.is_ascii_hexdigit() {
                        self.advance();
                    } else {
                        return Token {
                            kind: TokenKind::Unknown,
                            span: Span::new(start, self.current_index),
                        };
                    }
                }

                return Token {
                    kind: TokenKind::ByteValue,
                    span: Span::new(start, self.current_index),
                };
            }
        }

        loop {
            let byte = self.current_byte();

            if byte == b'.' {
                if matches!(
                    self.next_byte(),
                    Some(b'0'..=b'9' | b'e' | b'E' | b'+' | b'-' | b'_')
                ) {
                    if !is_float {
                        self.advance(); // Consume the dot
                    }

                    is_float = true;

                    self.advance();
                } else {
                    break;
                }

                loop {
                    let byte = self.current_byte();

                    if byte.is_ascii_digit() || byte == b'_' {
                        self.advance();

                        continue;
                    }

                    let next_byte = self.next_byte();

                    match (byte, next_byte) {
                        (b'e' | b'E', Some(b'0'..=b'9')) => {
                            self.advance();
                            self.advance();

                            continue;
                        }
                        (b'e' | b'E', Some(b'+' | b'-')) => {
                            self.advance();
                            self.advance();

                            continue;
                        }
                        _ => break,
                    }
                }
            }

            if byte.is_ascii_digit() || byte == b'_' {
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            Token {
                kind: TokenKind::FloatValue,
                span: Span::new(start, self.current_index),
            }
        } else {
            Token {
                kind: TokenKind::IntegerValue,
                span: Span::new(start, self.current_index),
            }
        }
    }

    #[inline]
    fn lex_keyword_or_identifier(&mut self) -> Token {
        let start = self.current_index;

        loop {
            let byte = self.current_byte();

            if byte.is_ascii_alphanumeric() || byte == b'_' {
                self.advance();
            } else {
                break;
            }
        }

        let kind = match &self.source[start..self.current_index] {
            b"Infinity" => TokenKind::FloatValue,
            b"NaN" => TokenKind::FloatValue,
            b"any" => TokenKind::Any,
            b"async" => TokenKind::Async,
            b"bool" => TokenKind::Bool,
            b"break" => TokenKind::Break,
            b"byte" => TokenKind::Byte,
            b"cell" => TokenKind::Cell,
            b"char" => TokenKind::Char,
            b"const" => TokenKind::Const,
            b"else" => TokenKind::Else,
            b"false" => TokenKind::FalseValue,
            b"float" => TokenKind::Float,
            b"fn" => TokenKind::Fn,
            b"if" => TokenKind::If,
            b"int" => TokenKind::Int,
            b"let" => TokenKind::Let,
            b"list" => TokenKind::List,
            b"loop" => TokenKind::Loop,
            b"map" => TokenKind::Map,
            b"mod" => TokenKind::Mod,
            b"mut" => TokenKind::Mut,
            b"pub" => TokenKind::Pub,
            b"return" => TokenKind::Return,
            b"str" => TokenKind::Str,
            b"struct" => TokenKind::Struct,
            b"true" => TokenKind::TrueValue,
            b"use" => TokenKind::Use,
            b"while" => TokenKind::While,
            _ => TokenKind::Identifier,
        };

        Token {
            kind,
            span: Span::new(start, self.current_index),
        }
    }

    #[inline]
    fn lex_string(&mut self) -> Token {
        let start = self.current_index;

        self.advance();

        while self.current_byte() != b'"' && !self.is_at_eof() {
            self.advance();
        }

        self.advance();

        Token {
            kind: TokenKind::StringValue,
            span: Span::new(start, self.current_index),
        }
    }

    #[inline]
    fn lex_character(&mut self) -> Token {
        let start = self.current_index;

        self.advance();

        if self.current_byte() < 128 {
            self.advance();
        } else {
            let character_start = self.current_index;

            while self.current_byte() >= 128 {
                self.advance();

                if self.current_index - character_start > 4 {
                    return Token {
                        kind: TokenKind::Unknown,
                        span: Span::new(start, self.current_index),
                    };
                }
            }

            let character_bytes = &self.source[character_start..self.current_index];

            match str::from_utf8(character_bytes) {
                Ok(_) => {}
                Err(_) => {
                    return Token {
                        kind: TokenKind::Unknown,
                        span: Span::new(start, self.current_index),
                    };
                }
            }
        }

        if self.current_byte() == b'\'' {
            self.advance();

            Token {
                kind: TokenKind::CharacterValue,
                span: Span::new(start, self.current_index),
            }
        } else {
            Token {
                kind: TokenKind::Unknown,
                span: Span::new(start, self.current_index),
            }
        }
    }

    #[inline]
    fn lex_plus(&mut self) -> Token {
        let start = self.current_index;

        self.advance();

        if self.current_byte() == b'=' {
            self.advance();

            Token {
                kind: TokenKind::PlusEqual,
                span: Span::new(start, self.current_index),
            }
        } else {
            Token {
                kind: TokenKind::Plus,
                span: Span::new(start, self.current_index),
            }
        }
    }

    #[inline]
    fn lex_minus(&mut self) -> Token {
        let start = self.current_index;

        if self.next_byte().is_some_and(|char| char.is_ascii_digit()) {
            return self.lex_numeric();
        }

        self.advance();

        match self.current_byte() {
            b'=' => {
                self.advance();

                return Token {
                    kind: TokenKind::MinusEqual,
                    span: Span::new(start, self.current_index),
                };
            }
            b'>' => {
                self.advance();

                return Token {
                    kind: TokenKind::ArrowThin,
                    span: Span::new(start, self.current_index),
                };
            }
            _ => {}
        }

        if self.peek_bytes(8) == b"Infinity" {
            self.current_index += 8;

            return Token {
                kind: TokenKind::FloatValue,
                span: Span::new(start, self.current_index),
            };
        }

        Token {
            kind: TokenKind::Minus,
            span: Span::new(start, self.current_index),
        }
    }

    #[inline]
    fn lex_asterisk(&mut self) -> Token {
        let start = self.current_index;

        self.advance();

        if self.current_byte() == b'=' {
            self.advance();

            Token {
                kind: TokenKind::AsteriskEqual,
                span: Span::new(start, self.current_index),
            }
        } else {
            Token {
                kind: TokenKind::Asterisk,
                span: Span::new(start, self.current_index),
            }
        }
    }

    #[inline]
    fn lex_slash(&mut self) -> Token {
        let start = self.current_index;

        self.advance();

        if self.current_byte() == b'=' {
            self.advance();

            Token {
                kind: TokenKind::SlashEqual,
                span: Span::new(start, self.current_index),
            }
        } else {
            Token {
                kind: TokenKind::Slash,
                span: Span::new(start, self.current_index),
            }
        }
    }

    #[inline]
    fn lex_percent(&mut self) -> Token {
        let start = self.current_index;

        self.advance();

        if self.current_byte() == b'=' {
            self.advance();

            Token {
                kind: TokenKind::PercentEqual,
                span: Span::new(start, self.current_index),
            }
        } else {
            Token {
                kind: TokenKind::Percent,
                span: Span::new(start, self.current_index),
            }
        }
    }

    #[inline]
    fn lex_exclamation_mark(&mut self) -> Token {
        let start = self.current_index;

        self.advance();

        if self.current_byte() == b'=' {
            self.advance();

            Token {
                kind: TokenKind::BangEqual,
                span: Span::new(start, self.current_index),
            }
        } else {
            Token {
                kind: TokenKind::Bang,
                span: Span::new(start, self.current_index),
            }
        }
    }

    #[inline]
    fn lex_equal(&mut self) -> Token {
        let start = self.current_index;

        self.advance();

        if self.current_byte() == b'=' {
            self.advance();

            Token {
                kind: TokenKind::DoubleEqual,
                span: Span::new(start, self.current_index),
            }
        } else {
            Token {
                kind: TokenKind::Equal,
                span: Span::new(start, self.current_index),
            }
        }
    }

    #[inline]
    fn lex_less_than(&mut self) -> Token {
        let start = self.current_index;

        self.advance();

        if self.current_byte() == b'=' {
            self.advance();

            Token {
                kind: TokenKind::LessEqual,
                span: Span::new(start, self.current_index),
            }
        } else {
            Token {
                kind: TokenKind::Less,
                span: Span::new(start, self.current_index),
            }
        }
    }

    #[inline]
    fn lex_greater_than(&mut self) -> Token {
        let start = self.current_index;

        self.advance();

        if self.current_byte() == b'=' {
            self.advance();

            Token {
                kind: TokenKind::GreaterEqual,
                span: Span::new(start, self.current_index),
            }
        } else {
            Token {
                kind: TokenKind::Greater,
                span: Span::new(start, self.current_index),
            }
        }
    }

    #[inline]
    fn lex_ampersand(&mut self) -> Token {
        let start = self.current_index;

        self.advance();

        if self.current_byte() == b'&' {
            self.advance();

            Token {
                kind: TokenKind::DoubleAmpersand,
                span: Span::new(start, self.current_index),
            }
        } else {
            Token {
                kind: TokenKind::Unknown,
                span: Span::new(start, self.current_index),
            }
        }
    }

    #[inline]
    fn lex_pipe(&mut self) -> Token {
        let start = self.current_index;

        self.advance();

        if self.current_byte() == b'|' {
            self.advance();

            Token {
                kind: TokenKind::DoublePipe,
                span: Span::new(start, self.current_index),
            }
        } else {
            Token {
                kind: TokenKind::Unknown,
                span: Span::new(start, self.current_index),
            }
        }
    }

    #[inline]
    fn lex_dot(&mut self) -> Token {
        let start = self.current_index;

        self.advance();

        if self.current_byte() == b'.' {
            self.advance();

            Token {
                kind: TokenKind::DoubleDot,
                span: Span::new(start, self.current_index),
            }
        } else {
            Token {
                kind: TokenKind::Dot,
                span: Span::new(start, self.current_index),
            }
        }
    }

    #[inline]
    fn lex_colon(&mut self) -> Token {
        let start = self.current_index;

        self.advance();

        if self.current_byte() == b':' {
            self.advance();

            Token {
                kind: TokenKind::DoubleColon,
                span: Span::new(start, self.current_index),
            }
        } else {
            Token {
                kind: TokenKind::Colon,
                span: Span::new(start, self.current_index),
            }
        }
    }

    #[inline]
    fn lex_carriage_return(&mut self) -> Token {
        let start = self.current_index;

        self.advance();

        if self.current_byte() == b'\n' {
            self.advance();
        }

        Token {
            kind: TokenKind::Newline,
            span: Span::new(start, self.current_index),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn character() {
//         let input = "'a'";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Character('a'), Span(0, 3)),
//                 (Token::Eof, Span(3, 3)),
//             ])
//         );
//     }

//     #[test]
//     fn map_expression() {
//         let input = "map { x = \"1\", y = 2, z = 3.0 }";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Map, Span(0, 3)),
//                 (Token::LeftBrace, Span(4, 5)),
//                 (Token::Identifier("x"), Span(6, 7)),
//                 (Token::Equal, Span(8, 9)),
//                 (Token::String("1"), Span(10, 13)),
//                 (Token::Comma, Span(13, 14)),
//                 (Token::Identifier("y"), Span(15, 16)),
//                 (Token::Equal, Span(17, 18)),
//                 (Token::Integer("2"), Span(19, 20)),
//                 (Token::Comma, Span(20, 21)),
//                 (Token::Identifier("z"), Span(22, 23)),
//                 (Token::Equal, Span(24, 25)),
//                 (Token::Float("3.0"), Span(26, 29)),
//                 (Token::RightBrace, Span(30, 31)),
//                 (Token::Eof, Span(31, 31)),
//             ])
//         );
//     }

//     #[test]
//     fn let_statement() {
//         let input = "let x = 42";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Let, Span(0, 3)),
//                 (Token::Identifier("x"), Span(4, 5)),
//                 (Token::Equal, Span(6, 7)),
//                 (Token::Integer("42"), Span(8, 10)),
//                 (Token::Eof, Span(10, 10)),
//             ])
//         );
//     }

//     #[test]
//     fn unit_struct() {
//         let input = "struct Foo";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Struct, Span(0, 6)),
//                 (Token::Identifier("Foo"), Span(7, 10)),
//                 (Token::Eof, Span(10, 10)),
//             ])
//         );
//     }

//     #[test]
//     fn tuple_struct() {
//         let input = "struct Foo(int, float)";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Struct, Span(0, 6)),
//                 (Token::Identifier("Foo"), Span(7, 10)),
//                 (Token::LeftParenthesis, Span(10, 11)),
//                 (Token::Int, Span(11, 14)),
//                 (Token::Comma, Span(14, 15)),
//                 (Token::FloatKeyword, Span(16, 21)),
//                 (Token::RightParenthesis, Span(21, 22)),
//                 (Token::Eof, Span(22, 22))
//             ])
//         );
//     }

//     #[test]
//     fn fields_struct() {
//         let input = "struct FooBar { foo: int, bar: float }";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Struct, Span(0, 6)),
//                 (Token::Identifier("FooBar"), Span(7, 13)),
//                 (Token::LeftBrace, Span(14, 15)),
//                 (Token::Identifier("foo"), Span(16, 19)),
//                 (Token::Colon, Span(19, 20)),
//                 (Token::Int, Span(21, 24)),
//                 (Token::Comma, Span(24, 25)),
//                 (Token::Identifier("bar"), Span(26, 29)),
//                 (Token::Colon, Span(29, 30)),
//                 (Token::FloatKeyword, Span(31, 36)),
//                 (Token::RightBrace, Span(37, 38)),
//                 (Token::Eof, Span(38, 38))
//             ])
//         );
//     }

//     #[test]
//     fn list_index() {
//         let input = "[1, 2, 3][1]";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::LeftBracket, Span(0, 1)),
//                 (Token::Integer("1"), Span(1, 2)),
//                 (Token::Comma, Span(2, 3)),
//                 (Token::Integer("2"), Span(4, 5)),
//                 (Token::Comma, Span(5, 6)),
//                 (Token::Integer("3"), Span(7, 8)),
//                 (Token::RightBracket, Span(8, 9)),
//                 (Token::LeftBracket, Span(9, 10)),
//                 (Token::Integer("1"), Span(10, 11)),
//                 (Token::RightBracket, Span(11, 12)),
//                 (Token::Eof, Span(12, 12)),
//             ])
//         )
//     }

//     #[test]
//     fn list() {
//         let input = "[1, 2, 3]";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::LeftBracket, Span(0, 1)),
//                 (Token::Integer("1"), Span(1, 2)),
//                 (Token::Comma, Span(2, 3)),
//                 (Token::Integer("2"), Span(4, 5)),
//                 (Token::Comma, Span(5, 6)),
//                 (Token::Integer("3"), Span(7, 8)),
//                 (Token::RightBracket, Span(8, 9)),
//                 (Token::Eof, Span(9, 9)),
//             ])
//         )
//     }

//     #[test]
//     fn map_field_access() {
//         let input = "{a = 1, b = 2, c = 3}.c";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::LeftBrace, Span(0, 1)),
//                 (Token::Identifier("a"), Span(1, 2)),
//                 (Token::Equal, Span(3, 4)),
//                 (Token::Integer("1"), Span(5, 6)),
//                 (Token::Comma, Span(6, 7)),
//                 (Token::Identifier("b"), Span(8, 9)),
//                 (Token::Equal, Span(10, 11)),
//                 (Token::Integer("2"), Span(12, 13)),
//                 (Token::Comma, Span(13, 14)),
//                 (Token::Identifier("c"), Span(15, 16)),
//                 (Token::Equal, Span(17, 18)),
//                 (Token::Integer("3"), Span(19, 20)),
//                 (Token::RightBrace, Span(20, 21)),
//                 (Token::Dot, Span(21, 22)),
//                 (Token::Identifier("c"), Span(22, 23)),
//                 (Token::Eof, Span(23, 23)),
//             ])
//         )
//     }

//     #[test]
//     fn range() {
//         let input = "0..42";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("0"), Span(0, 1)),
//                 (Token::DoubleDot, Span(1, 3)),
//                 (Token::Integer("42"), Span(3, 5)),
//                 (Token::Eof, Span(5, 5))
//             ])
//         );
//     }

//     #[test]
//     fn negate_expression() {
//         let input = "x = -42; -x";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Identifier("x"), Span(0, 1)),
//                 (Token::Equal, Span(2, 3)),
//                 (Token::Integer("-42"), Span(4, 7)),
//                 (Token::Semicolon, Span(7, 8)),
//                 (Token::Minus, Span(9, 10)),
//                 (Token::Identifier("x"), Span(10, 11)),
//                 (Token::Eof, Span(11, 11))
//             ])
//         );
//     }

//     #[test]
//     fn not_expression() {
//         let input = "!true; !false";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Bang, Span(0, 1)),
//                 (Token::Boolean("true"), Span(1, 5)),
//                 (Token::Semicolon, Span(5, 6)),
//                 (Token::Bang, Span(7, 8)),
//                 (Token::Boolean("false"), Span(8, 13)),
//                 (Token::Eof, Span(13, 13))
//             ])
//         );
//     }

//     #[test]
//     fn if_else() {
//         let input = "if x < 10 { x + 1 } else { x }";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::If, Span(0, 2)),
//                 (Token::Identifier("x"), Span(3, 4)),
//                 (Token::Less, Span(5, 6)),
//                 (Token::Integer("10"), Span(7, 9)),
//                 (Token::LeftBrace, Span(10, 11)),
//                 (Token::Identifier("x"), Span(12, 13)),
//                 (Token::Plus, Span(14, 15)),
//                 (Token::Integer("1"), Span(16, 17)),
//                 (Token::RightBrace, Span(18, 19)),
//                 (Token::Else, Span(20, 24)),
//                 (Token::LeftBrace, Span(25, 26)),
//                 (Token::Identifier("x"), Span(27, 28)),
//                 (Token::RightBrace, Span(29, 30)),
//                 (Token::Eof, Span(30, 30)),
//             ])
//         )
//     }

//     #[test]
//     fn while_loop() {
//         let input = "while x < 10 { x += 1 }";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::While, Span(0, 5)),
//                 (Token::Identifier("x"), Span(6, 7)),
//                 (Token::Less, Span(8, 9)),
//                 (Token::Integer("10"), Span(10, 12)),
//                 (Token::LeftBrace, Span(13, 14)),
//                 (Token::Identifier("x"), Span(15, 16)),
//                 (Token::PlusEqual, Span(17, 19)),
//                 (Token::Integer("1"), Span(20, 21)),
//                 (Token::RightBrace, Span(22, 23)),
//                 (Token::Eof, Span(23, 23)),
//             ])
//         )
//     }

//     #[test]
//     fn add_assign() {
//         let input = "x += 42";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Identifier("x"), Span(0, 1)),
//                 (Token::PlusEqual, Span(2, 4)),
//                 (Token::Integer("42"), Span(5, 7)),
//                 (Token::Eof, Span(7, 7)),
//             ])
//         )
//     }

//     #[test]
//     fn or() {
//         let input = "true || false";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Boolean("true"), Span(0, 4)),
//                 (Token::DoublePipe, Span(5, 7)),
//                 (Token::Boolean("false"), Span(8, 13)),
//                 (Token::Eof, Span(13, 13)),
//             ])
//         )
//     }

//     #[test]
//     fn block() {
//         let input = "{ x = 42; y = \"foobar\" }";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::LeftBrace, Span(0, 1)),
//                 (Token::Identifier("x"), Span(2, 3)),
//                 (Token::Equal, Span(4, 5)),
//                 (Token::Integer("42"), Span(6, 8)),
//                 (Token::Semicolon, Span(8, 9)),
//                 (Token::Identifier("y"), Span(10, 11)),
//                 (Token::Equal, Span(12, 13)),
//                 (Token::String("foobar"), Span(14, 22)),
//                 (Token::RightBrace, Span(23, 24)),
//                 (Token::Eof, Span(24, 24)),
//             ])
//         )
//     }

//     #[test]
//     fn not_equal() {
//         let input = "42 != 42";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("42"), Span(0, 2)),
//                 (Token::BangEqual, Span(3, 5)),
//                 (Token::Integer("42"), Span(6, 8)),
//                 (Token::Eof, Span(8, 8)),
//             ])
//         )
//     }

//     #[test]
//     fn equal() {
//         let input = "42 == 42";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("42"), Span(0, 2)),
//                 (Token::DoubleEqual, Span(3, 5)),
//                 (Token::Integer("42"), Span(6, 8)),
//                 (Token::Eof, Span(8, 8)),
//             ])
//         )
//     }

//     #[test]
//     fn modulo() {
//         let input = "42 % 2";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("42"), Span(0, 2)),
//                 (Token::Percent, Span(3, 4)),
//                 (Token::Integer("2"), Span(5, 6)),
//                 (Token::Eof, Span(6, 6)),
//             ])
//         )
//     }

//     #[test]
//     fn divide() {
//         let input = "42 / 2";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("42"), Span(0, 2)),
//                 (Token::Slash, Span(3, 4)),
//                 (Token::Integer("2"), Span(5, 6)),
//                 (Token::Eof, Span(6, 6)),
//             ])
//         )
//     }

//     #[test]
//     fn greater_than() {
//         let input = ">";

//         assert_eq!(
//             lex(input),
//             Ok(vec![(Token::Greater, Span(0, 1)), (Token::Eof, Span(1, 1))])
//         )
//     }

//     #[test]
//     fn greater_than_or_equal() {
//         let input = ">=";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::GreaterEqual, Span(0, 2)),
//                 (Token::Eof, Span(2, 2))
//             ])
//         )
//     }

//     #[test]
//     fn less_than() {
//         let input = "<";

//         assert_eq!(
//             lex(input),
//             Ok(vec![(Token::Less, Span(0, 1)), (Token::Eof, Span(1, 1))])
//         )
//     }

//     #[test]
//     fn less_than_or_equal() {
//         let input = "<=";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::LessEqual, Span(0, 2)),
//                 (Token::Eof, Span(2, 2))
//             ])
//         )
//     }

//     #[test]
//     fn infinity() {
//         let input = "Infinity";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Float("Infinity"), Span(0, 8)),
//                 (Token::Eof, Span(8, 8)),
//             ])
//         )
//     }

//     #[test]
//     fn negative_infinity() {
//         let input = "-Infinity";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Float("-Infinity"), Span(0, 9)),
//                 (Token::Eof, Span(9, 9)),
//             ])
//         )
//     }

//     #[test]
//     fn nan() {
//         let input = "NaN";

//         assert!(lex(input).is_ok_and(|tokens| tokens[0].0 == Token::Float("NaN")));
//     }

//     #[test]
//     fn complex_float() {
//         let input = "42.42e42";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Float("42.42e42"), Span(0, 8)),
//                 (Token::Eof, Span(8, 8)),
//             ])
//         )
//     }

//     #[test]
//     fn max_integer() {
//         let input = "9223372036854775807";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("9223372036854775807"), Span(0, 19)),
//                 (Token::Eof, Span(19, 19)),
//             ])
//         )
//     }

//     #[test]
//     fn min_integer() {
//         let input = "-9223372036854775808";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("-9223372036854775808"), Span(0, 20)),
//                 (Token::Eof, Span(20, 20)),
//             ])
//         )
//     }

//     #[test]
//     fn subtract_negative_integers() {
//         let input = "-42 - -42";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("-42"), Span(0, 3)),
//                 (Token::Minus, Span(4, 5)),
//                 (Token::Integer("-42"), Span(6, 9)),
//                 (Token::Eof, Span(9, 9)),
//             ])
//         )
//     }

//     #[test]
//     fn negative_integer() {
//         let input = "-42";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("-42"), Span(0, 3)),
//                 (Token::Eof, Span(3, 3))
//             ])
//         )
//     }

//     #[test]
//     fn read_line() {
//         let input = "read_line()";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Identifier("read_line"), Span(0, 9)),
//                 (Token::LeftParenthesis, Span(9, 10)),
//                 (Token::RightParenthesis, Span(10, 11)),
//                 (Token::Eof, Span(11, 11)),
//             ])
//         )
//     }

//     #[test]
//     fn write_line() {
//         let input = "write_line(\"Hello, world!\")";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Identifier("write_line"), Span(0, 10)),
//                 (Token::LeftParenthesis, Span(10, 11)),
//                 (Token::String("Hello, world!"), Span(11, 26)),
//                 (Token::RightParenthesis, Span(26, 27)),
//                 (Token::Eof, Span(27, 27)),
//             ])
//         )
//     }

//     #[test]
//     fn string_concatenation() {
//         let input = "\"Hello, \" + \"world!\"";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::String("Hello, "), Span(0, 9)),
//                 (Token::Plus, Span(10, 11)),
//                 (Token::String("world!"), Span(12, 20)),
//                 (Token::Eof, Span(20, 20)),
//             ])
//         )
//     }

//     #[test]
//     fn string() {
//         let input = "\"Hello, world!\"";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::String("Hello, world!"), Span(0, 15)),
//                 (Token::Eof, Span(15, 15)),
//             ])
//         )
//     }

//     #[test]
//     fn r#true() {
//         let input = "true";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Boolean("true"), Span(0, 4)),
//                 (Token::Eof, Span(4, 4)),
//             ])
//         )
//     }

//     #[test]
//     fn r#false() {
//         let input = "false";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Boolean("false"), Span(0, 5)),
//                 (Token::Eof, Span(5, 5))
//             ])
//         )
//     }

//     #[test]
//     fn property_access_function_call() {
//         let input = "42.is_even()";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("42"), Span(0, 2)),
//                 (Token::Dot, Span(2, 3)),
//                 (Token::Identifier("is_even"), Span(3, 10)),
//                 (Token::LeftParenthesis, Span(10, 11)),
//                 (Token::RightParenthesis, Span(11, 12)),
//                 (Token::Eof, Span(12, 12)),
//             ])
//         )
//     }

//     #[test]
//     fn empty() {
//         let input = "";

//         assert_eq!(lex(input), Ok(vec![(Token::Eof, Span(0, 0))]))
//     }

//     #[test]
//     fn reserved_identifier() {
//         let input = "length";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Identifier("length"), Span(0, 6)),
//                 (Token::Eof, Span(6, 6)),
//             ])
//         )
//     }

//     #[test]
//     fn square_braces() {
//         let input = "[]";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::LeftBracket, Span(0, 1)),
//                 (Token::RightBracket, Span(1, 2)),
//                 (Token::Eof, Span(2, 2)),
//             ])
//         )
//     }

//     #[test]
//     fn small_float() {
//         let input = "1.23";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Float("1.23"), Span(0, 4)),
//                 (Token::Eof, Span(4, 4)),
//             ])
//         )
//     }

//     #[test]
//     #[allow(clippy::excessive_precision)]
//     fn big_float() {
//         let input = "123456789.123456789";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Float("123456789.123456789"), Span(0, 19)),
//                 (Token::Eof, Span(19, 19)),
//             ])
//         )
//     }

//     #[test]
//     fn float_with_exponent() {
//         let input = "1.23e4";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Float("1.23e4"), Span(0, 6)),
//                 (Token::Eof, Span(6, 6)),
//             ])
//         )
//     }

//     #[test]
//     fn float_with_negative_exponent() {
//         let input = "1.23e-4";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Float("1.23e-4"), Span(0, 7)),
//                 (Token::Eof, Span(7, 7)),
//             ])
//         )
//     }

//     #[test]
//     fn float_infinity_and_nan() {
//         let input = "Infinity -Infinity NaN";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Float("Infinity"), Span(0, 8)),
//                 (Token::Float("-Infinity"), Span(9, 18)),
//                 (Token::Float("NaN"), Span(19, 22)),
//                 (Token::Eof, Span(22, 22)),
//             ])
//         )
//     }

//     #[test]
//     fn add() {
//         let input = "1 + 2";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("1"), Span(0, 1)),
//                 (Token::Plus, Span(2, 3)),
//                 (Token::Integer("2"), Span(4, 5)),
//                 (Token::Eof, Span(5, 5)),
//             ])
//         )
//     }

//     #[test]
//     fn multiply() {
//         let input = "1 * 2";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("1"), Span(0, 1)),
//                 (Token::Star, Span(2, 3)),
//                 (Token::Integer("2"), Span(4, 5)),
//                 (Token::Eof, Span(5, 5)),
//             ])
//         )
//     }

//     #[test]
//     fn add_and_multiply() {
//         let input = "1 + 2 * 3";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("1"), Span(0, 1)),
//                 (Token::Plus, Span(2, 3)),
//                 (Token::Integer("2"), Span(4, 5)),
//                 (Token::Star, Span(6, 7)),
//                 (Token::Integer("3"), Span(8, 9)),
//                 (Token::Eof, Span(9, 9)),
//             ])
//         );
//     }

//     #[test]
//     fn assignment() {
//         let input = "a = 1 + 2 * 3";

//         assert_eq!(
//             lex(input,),
//             Ok(vec![
//                 (Token::Identifier("a"), Span(0, 1)),
//                 (Token::Equal, Span(2, 3)),
//                 (Token::Integer("1"), Span(4, 5)),
//                 (Token::Plus, Span(6, 7)),
//                 (Token::Integer("2"), Span(8, 9)),
//                 (Token::Star, Span(10, 11)),
//                 (Token::Integer("3"), Span(12, 13)),
//                 (Token::Eof, Span(13, 13)),
//             ])
//         );
//     }
// }
