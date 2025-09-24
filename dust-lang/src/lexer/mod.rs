mod validate_utf8;

#[cfg(test)]
mod tests;

pub use validate_utf8::validate_utf8_and_find_token_starts;

use crate::{Span, TokenKind, token::Token};

#[derive(Debug, Default)]
pub struct Lexer<'src> {
    /// Dust source code as utf-8 bytes.
    source: &'src [u8],

    /// Index into `token_starts`.
    next_token_start: usize,

    /// Locations of the start of each token in the source.
    token_starts: Vec<usize>,

    /// Whether we've reached the end of the file.
    is_eof: bool,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src [u8]) -> Option<Self> {
        let (is_valid_utf8, token_starts) = validate_utf8_and_find_token_starts(source);

        if is_valid_utf8 {
            Some(Lexer {
                source,
                next_token_start: 0,
                token_starts,
                is_eof: false,
            })
        } else {
            None
        }
    }

    pub fn source(&self) -> &'src [u8] {
        self.source
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_eof {
            return None;
        } else if self.next_token_start >= self.token_starts.len() {
            self.is_eof = true;
            let length = self.source.len() as u32;

            return Some(Token {
                kind: TokenKind::Eof,
                span: Span(length, length),
            });
        }

        let start_index = *self.token_starts.get(self.next_token_start)?;

        self.next_token_start += 1;

        let end_index = self
            .token_starts
            .get(self.next_token_start)
            .map(|i| i.saturating_sub(1))
            .unwrap_or(self.source.len());
        let word = self.source.get(start_index..end_index)?;
        let token_kind = match word {
            // Keywords
            b"any" => TokenKind::Any,
            b"async" => TokenKind::Async,
            b"bool" => TokenKind::Bool,
            b"break" => TokenKind::Break,
            b"byte" => TokenKind::Byte,
            b"cell" => TokenKind::Cell,
            b"char" => TokenKind::Char,
            b"const" => TokenKind::Const,
            b"else" => TokenKind::Else,
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
            b"use" => TokenKind::Use,
            b"while" => TokenKind::While,

            // Operators and punctuation
            b"->" => TokenKind::ArrowThin,
            b"*" => TokenKind::Asterisk,
            b"*=" => TokenKind::AsteriskEqual,
            b"!=" => TokenKind::BangEqual,
            b"!" => TokenKind::Bang,
            b":" => TokenKind::Colon,
            b"," => TokenKind::Comma,
            b"." => TokenKind::Dot,
            b"&&" => TokenKind::DoubleAmpersand,
            b"::" => TokenKind::DoubleColon,
            b".." => TokenKind::DoubleDot,
            b"==" => TokenKind::DoubleEqual,
            b"||" => TokenKind::DoublePipe,
            b"=" => TokenKind::Equal,
            b">" => TokenKind::Greater,
            b">=" => TokenKind::GreaterEqual,
            b"{" => TokenKind::LeftCurlyBrace,
            b"[" => TokenKind::LeftSquareBracket,
            b"(" => TokenKind::LeftParenthesis,
            b"<" => TokenKind::Less,
            b"<=" => TokenKind::LessEqual,
            b"-" => TokenKind::Minus,
            b"-=" => TokenKind::MinusEqual,
            b"%" => TokenKind::Percent,
            b"%=" => TokenKind::PercentEqual,
            b"+" => TokenKind::Plus,
            b"+=" => TokenKind::PlusEqual,
            b"}" => TokenKind::RightCurlyBrace,
            b"]" => TokenKind::RightSquareBracket,
            b")" => TokenKind::RightParenthesis,
            b";" => TokenKind::Semicolon,
            b"/" => TokenKind::Slash,
            b"/=" => TokenKind::SlashEqual,

            // Literals
            b"true" => TokenKind::TrueValue,
            b"false" => TokenKind::FalseValue,
            b"Infinity" => TokenKind::FloatValue,
            b"-Infinity" => TokenKind::FloatValue,
            _ if word[0].is_ascii_alphabetic() || word[0] == b'_' => {
                if word.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'_') {
                    TokenKind::Identifier
                } else {
                    TokenKind::Unknown
                }
            }
            _ if word.starts_with(b"0x")
                && word.len() > 2
                && word[2..].iter().all(|b| b.is_ascii_hexdigit()) =>
            {
                TokenKind::ByteValue
            }
            _ if word[0].is_ascii_digit() => {
                let mut chars = word.iter().peekable();
                let mut has_decimal = false;
                let mut has_exponent = false;

                while let Some(&b) = chars.peek() {
                    match b {
                        b'0'..=b'9' => {
                            chars.next();
                        }
                        b'.' if !has_decimal => {
                            has_decimal = true;
                            chars.next();
                        }
                        b'e' | b'E' => {
                            if !has_exponent && has_decimal {
                                has_exponent = true;
                                has_decimal = true;
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        _ => break,
                    }
                }

                if chars.next().is_none() {
                    if has_decimal {
                        TokenKind::FloatValue
                    } else {
                        TokenKind::IntegerValue
                    }
                } else {
                    TokenKind::Unknown
                }
            }
            _ if word.starts_with(b"\"") && word.ends_with(b"\"") && word.len() >= 2 => {
                TokenKind::StringValue
            }
            _ if word.starts_with(b"'") && word.ends_with(b"'") && word.len() == 3 => {
                TokenKind::CharacterValue
            }
            _ => TokenKind::Unknown,
        };

        Some(Token {
            kind: token_kind,
            span: Span(start_index as u32, end_index as u32),
        })
    }
}
