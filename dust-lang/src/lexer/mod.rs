#[cfg(test)]
mod tests;

use crate::{
    Span,
    token::{Token, TokenKind},
};

#[derive(Debug)]
pub struct Lexer<'src> {
    source: &'src [u8],
    index: usize,
    token_start: Option<usize>,
    token_flags: TokenFlags,
    is_eof_or_error: bool,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src [u8]) -> Self {
        Self {
            source,
            index: 0,
            token_start: None,
            token_flags: TokenFlags::default(),
            is_eof_or_error: false,
        }
    }

    #[inline]
    pub fn source(&self) -> &'src [u8] {
        self.source
    }

    #[inline]
    fn len(&self) -> usize {
        self.source.len()
    }

    #[inline(always)]
    fn finish_token(&mut self) -> Option<Token> {
        if let Some(start) = self.token_start.take() {
            let end = self.index;

            if end > start {
                let span = Span(start as u32, end as u32);
                let word = &self.source[start..end];
                let kind = if self.token_flags.starts_with_digit {
                    if self.token_flags.in_hexadecimal {
                        if !self.token_flags.unknown && self.token_flags.hex_digits > 0 {
                            TokenKind::ByteValue
                        } else {
                            TokenKind::Unknown
                        }
                    } else if !self.token_flags.unknown {
                        if self.token_flags.has_decimal {
                            TokenKind::FloatValue
                        } else {
                            TokenKind::IntegerValue
                        }
                    } else {
                        TokenKind::Unknown
                    }
                } else if !word.is_empty() && (word[0].is_ascii_alphabetic() || word[0] == b'_') {
                    if !self.token_flags.saw_non_ascii {
                        if let Some(kind) = keyword_kind(word) {
                            kind
                        } else {
                            TokenKind::Identifier
                        }
                    } else {
                        TokenKind::Unknown
                    }
                } else {
                    TokenKind::Unknown
                };

                self.token_flags = TokenFlags::default();

                return Some(Token { kind, span });
            }
        }
        None
    }

    #[inline(always)]
    fn scan_utf8_sequence(&self, start: usize) -> Result<usize, usize> {
        let input = self.source;
        let first = input[start];

        if first < 0x80 {
            return Ok(1);
        }

        let width = utf8_char_width(first);

        if width == 0 || start + width > input.len() {
            return Err(start);
        }

        match width {
            2 => {
                let second = input[start + 1];

                if (second as i8) >= -64 {
                    return Err(start);
                }
            }
            3 => {
                let second = input[start + 1];

                match (first, second) {
                    (0xE0, 0xA0..=0xBF)
                    | (0xE1..=0xEC, 0x80..=0xBF)
                    | (0xED, 0x80..=0x9F)
                    | (0xEE..=0xEF, 0x80..=0xBF) => {}
                    _ => return Err(start),
                }
                let third = input[start + 2];

                if (third as i8) >= -64 {
                    return Err(start);
                }
            }
            4 => {
                let second = input[start + 1];

                match (first, second) {
                    (0xF0, 0x90..=0xBF) | (0xF1..=0xF3, 0x80..=0xBF) | (0xF4, 0x80..=0x8F) => {}
                    _ => return Err(start),
                }
                let third = input[start + 2];

                if (third as i8) >= -64 {
                    return Err(start);
                }

                let fourth = input[start + 3];

                if (fourth as i8) >= -64 {
                    return Err(start);
                }
            }
            _ => return Err(start),
        }

        Ok(width)
    }

    #[inline(always)]
    fn scan_string(&mut self) -> Result<Option<Token>, usize> {
        let start = self.index;

        if self.source[start] != b'"' {
            return Ok(None);
        }

        let mut index = start + 1;

        while index < self.len() {
            let byte = self.source[index];

            if byte < 0x80 {
                if byte == b'"' {
                    let end = index + 1;

                    self.index = end;

                    let span = Span(start as u32, end as u32);

                    return Ok(Some(Token {
                        kind: TokenKind::StringValue,
                        span,
                    }));
                } else {
                    index += 1;
                }
            } else {
                match self.scan_utf8_sequence(index) {
                    Ok(width) => index += width,
                    Err(err_index) => return Err(err_index),
                }
            }
        }

        let end = self.len();

        self.index = end;

        let span = Span(start as u32, end as u32);

        Ok(Some(Token {
            kind: TokenKind::StringValue,
            span,
        }))
    }

    #[inline(always)]
    fn scan_chararacter(&mut self) -> Result<Option<Token>, usize> {
        let start = self.index;

        if self.source[start] != b'\'' {
            return Ok(None);
        }

        let mut index = start + 1;

        while index < self.len() {
            let byte = self.source[index];

            if byte < 0x80 {
                if byte == b'\'' {
                    let end = index + 1;

                    self.index = end;

                    let span = Span(start as u32, end as u32);

                    return Ok(Some(Token {
                        kind: TokenKind::CharacterValue,
                        span,
                    }));
                } else {
                    index += 1;
                }
            } else {
                match self.scan_utf8_sequence(index) {
                    Ok(width) => index += width,
                    Err(err_index) => return Err(err_index),
                }
            }
        }

        let end = self.len();

        self.index = end;

        let span = Span(start as u32, end as u32);

        Ok(Some(Token {
            kind: TokenKind::CharacterValue,
            span,
        }))
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token, usize>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.is_eof_or_error {
                return None;
            }

            if self.index >= self.len() {
                if let Some(token) = self.finish_token() {
                    return Some(Ok(token));
                }

                self.is_eof_or_error = true;

                let length = self.source.len() as u32;

                return Some(Ok(Token {
                    kind: TokenKind::Eof,
                    span: Span(length, length),
                }));
            }

            let byte = self.source[self.index];

            if byte < 0x80 {
                if is_ascii_whitespace(byte) {
                    if let Some(tok) = self.finish_token() {
                        return Some(Ok(tok));
                    }

                    self.index += 1;

                    continue;
                }

                if byte == b'"' {
                    if let Some(token) = self.finish_token() {
                        return Some(Ok(token));
                    }
                    match self.scan_string() {
                        Ok(Some(token)) => return Some(Ok(token)),
                        Ok(None) => {
                            self.index += 1;

                            continue;
                        }
                        Err(err_index) => {
                            self.is_eof_or_error = true;

                            return Some(Err(err_index));
                        }
                    }
                }

                if byte == b'\'' {
                    if let Some(token) = self.finish_token() {
                        return Some(Ok(token));
                    }

                    match self.scan_chararacter() {
                        Ok(Some(token)) => return Some(Ok(token)),
                        Ok(None) => {
                            self.index += 1;

                            continue;
                        }
                        Err(err_index) => {
                            self.is_eof_or_error = true;

                            return Some(Err(err_index));
                        }
                    }
                }

                if byte == b'.'
                    && let Some(start) = self.token_start
                {
                    let token_first = self.source[start];
                    let next_is_digit = (self.index + 1) < self.len() && {
                        let byte = self.source[self.index + 1];

                        byte.is_ascii_digit() || byte == b'_'
                    };

                    if token_first.is_ascii_digit() && next_is_digit {
                        self.index += 1;
                        self.token_flags.length += 1;
                        self.token_flags.has_decimal = true;

                        continue;
                    }
                }

                if is_operator_or_punctuation(byte) {
                    if let Some(token) = self.finish_token() {
                        return Some(Ok(token));
                    }

                    if byte == b'-' && self.index + 9 <= self.len() {
                        let slice = &self.source[self.index..self.index + 9];

                        if slice == b"-Infinity" {
                            let span = Span(self.index as u32, (self.index + 9) as u32);
                            let kind = TokenKind::FloatValue;
                            self.index += 9;

                            return Some(Ok(Token { kind, span }));
                        }
                    }

                    if self.index + 1 < self.len() {
                        let first_byte = self.source[self.index];
                        let second_byte = self.source[self.index + 1];

                        if let Some(two_kind) = classify_two_operator(first_byte, second_byte) {
                            let span = Span(self.index as u32, (self.index + 2) as u32);

                            self.index += 2;

                            return Some(Ok(Token {
                                kind: two_kind,
                                span,
                            }));
                        }
                    }

                    let span = Span(self.index as u32, (self.index + 1) as u32);
                    let kind = classify_single_operator(self.source[self.index]);
                    self.index += 1;

                    return Some(Ok(Token { kind, span }));
                }

                if self.token_start.is_none() {
                    self.token_start = Some(self.index);
                    self.token_flags = TokenFlags::start(byte);
                } else if self.token_flags.starts_with_digit {
                    let next = self.source.get(self.index + 1).copied();

                    self.token_flags.push(byte, next);
                }

                self.index += 1;

                continue;
            }

            match self.scan_utf8_sequence(self.index) {
                Ok(width) => {
                    if self.token_start.is_none() {
                        self.token_start = Some(self.index);
                        self.token_flags = TokenFlags::start(self.source[self.index]);
                    }

                    self.token_flags.saw_non_ascii = true;

                    if self.token_flags.starts_with_digit {
                        self.token_flags.unknown = true;
                    }

                    self.token_flags.length = self.token_flags.length.saturating_add(width);
                    self.index += width;

                    continue;
                }
                Err(err_index) => {
                    self.is_eof_or_error = true;

                    return Some(Err(err_index));
                }
            }
        }
    }
}

const WS_MASK: u128 = (1u128 << (b' ' as u32))
    | (1u128 << (b'\n' as u32))
    | (1u128 << (b'\r' as u32))
    | (1u128 << (b'\t' as u32));

const PUNCT_MASK: u128 = (1u128 << (b'!' as u32))
    | (1u128 << (b'"' as u32))
    | (1u128 << (b'#' as u32))
    | (1u128 << (b'$' as u32))
    | (1u128 << (b'%' as u32))
    | (1u128 << (b'&' as u32))
    | (1u128 << (b'\'' as u32))
    | (1u128 << (b'(' as u32))
    | (1u128 << (b')' as u32))
    | (1u128 << (b'*' as u32))
    | (1u128 << (b'+' as u32))
    | (1u128 << (b',' as u32))
    | (1u128 << (b'-' as u32))
    | (1u128 << (b'.' as u32))
    | (1u128 << (b'/' as u32))
    | (1u128 << (b':' as u32))
    | (1u128 << (b';' as u32))
    | (1u128 << (b'<' as u32))
    | (1u128 << (b'=' as u32))
    | (1u128 << (b'>' as u32))
    | (1u128 << (b'?' as u32))
    | (1u128 << (b'@' as u32))
    | (1u128 << (b'[' as u32))
    | (1u128 << (b'\\' as u32))
    | (1u128 << (b']' as u32))
    | (1u128 << (b'^' as u32))
    | (1u128 << (b'`' as u32))
    | (1u128 << (b'{' as u32))
    | (1u128 << (b'|' as u32))
    | (1u128 << (b'}' as u32))
    | (1u128 << (b'~' as u32));

#[inline(always)]
fn is_ascii_whitespace(byte: u8) -> bool {
    byte < 128 && ((WS_MASK >> (byte as u32)) & 1) != 0
}

#[inline(always)]
fn is_operator_or_punctuation(byte: u8) -> bool {
    byte < 128 && ((PUNCT_MASK >> (byte as u32)) & 1) != 0
}

#[inline(always)]
fn keyword_kind(word: &[u8]) -> Option<TokenKind> {
    match word.len() {
        2 => match word {
            b"fn" => Some(TokenKind::Fn),
            b"if" => Some(TokenKind::If),
            _ => None,
        },
        3 => match word {
            b"any" => Some(TokenKind::Any),
            b"int" => Some(TokenKind::Int),
            b"let" => Some(TokenKind::Let),
            b"map" => Some(TokenKind::Map),
            b"mod" => Some(TokenKind::Mod),
            b"mut" => Some(TokenKind::Mut),
            b"pub" => Some(TokenKind::Pub),
            b"str" => Some(TokenKind::Str),
            b"use" => Some(TokenKind::Use),
            _ => None,
        },
        4 => match word {
            b"bool" => Some(TokenKind::Bool),
            b"byte" => Some(TokenKind::Byte),
            b"cell" => Some(TokenKind::Cell),
            b"char" => Some(TokenKind::Char),
            b"else" => Some(TokenKind::Else),
            b"list" => Some(TokenKind::List),
            b"loop" => Some(TokenKind::Loop),
            b"true" => Some(TokenKind::TrueValue),
            _ => None,
        },
        5 => match word {
            b"async" => Some(TokenKind::Async),
            b"break" => Some(TokenKind::Break),
            b"const" => Some(TokenKind::Const),
            b"float" => Some(TokenKind::Float),
            b"while" => Some(TokenKind::While),
            b"false" => Some(TokenKind::FalseValue),
            _ => None,
        },
        6 => match word {
            b"return" => Some(TokenKind::Return),
            b"struct" => Some(TokenKind::Struct),
            _ => None,
        },
        8 => match word {
            b"Infinity" => Some(TokenKind::FloatValue),
            _ => None,
        },
        _ => None,
    }
}

#[inline(always)]
fn classify_two_operator(b1: u8, b2: u8) -> Option<TokenKind> {
    Some(match (b1, b2) {
        (b'-', b'>') => TokenKind::ArrowThin,
        (b'*', b'=') => TokenKind::AsteriskEqual,
        (b'!', b'=') => TokenKind::BangEqual,
        (b'&', b'&') => TokenKind::DoubleAmpersand,
        (b':', b':') => TokenKind::DoubleColon,
        (b'.', b'.') => TokenKind::DoubleDot,
        (b'=', b'=') => TokenKind::DoubleEqual,
        (b'|', b'|') => TokenKind::DoublePipe,
        (b'>', b'=') => TokenKind::GreaterEqual,
        (b'<', b'=') => TokenKind::LessEqual,
        (b'-', b'=') => TokenKind::MinusEqual,
        (b'%', b'=') => TokenKind::PercentEqual,
        (b'+', b'=') => TokenKind::PlusEqual,
        (b'/', b'=') => TokenKind::SlashEqual,
        _ => return None,
    })
}

#[inline(always)]
fn classify_single_operator(b: u8) -> TokenKind {
    match b {
        b'*' => TokenKind::Asterisk,
        b'!' => TokenKind::Bang,
        b':' => TokenKind::Colon,
        b',' => TokenKind::Comma,
        b'.' => TokenKind::Dot,
        b'=' => TokenKind::Equal,
        b'>' => TokenKind::Greater,
        b'{' => TokenKind::LeftCurlyBrace,
        b'[' => TokenKind::LeftSquareBracket,
        b'(' => TokenKind::LeftParenthesis,
        b'<' => TokenKind::Less,
        b'-' => TokenKind::Minus,
        b'%' => TokenKind::Percent,
        b'+' => TokenKind::Plus,
        b'}' => TokenKind::RightCurlyBrace,
        b']' => TokenKind::RightSquareBracket,
        b')' => TokenKind::RightParenthesis,
        b';' => TokenKind::Semicolon,
        b'/' => TokenKind::Slash,
        b'@' => TokenKind::Unknown,
        b'^' => TokenKind::Unknown,
        b'`' => TokenKind::Unknown,
        b'~' => TokenKind::Unknown,
        b'?' => TokenKind::Unknown,
        b'#' => TokenKind::Unknown,
        b'$' => TokenKind::Unknown,
        b'&' => TokenKind::Unknown,
        b'|' => TokenKind::Unknown,
        b'\\' => TokenKind::Unknown,
        b'"' => TokenKind::Unknown,
        b'\'' => TokenKind::Unknown,
        _ => TokenKind::Unknown,
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct TokenFlags {
    starts_with_digit: bool,
    in_hexadecimal: bool,
    hex_digits: usize,
    has_decimal: bool,
    has_exponent: bool,
    unknown: bool,
    saw_non_ascii: bool,
    length: usize,
    first_byte: u8,
}

impl TokenFlags {
    #[inline(always)]
    fn start(first: u8) -> Self {
        Self {
            starts_with_digit: first.is_ascii_digit(),
            in_hexadecimal: false,
            hex_digits: 0,
            has_decimal: false,
            has_exponent: false,
            unknown: false,
            saw_non_ascii: false,
            length: 1,
            first_byte: first,
        }
    }

    #[inline(always)]
    fn push(&mut self, b: u8, next: Option<u8>) {
        self.length += 1;

        if self.in_hexadecimal {
            if b.is_ascii_hexdigit() {
                self.hex_digits += 1;
            } else if b == b'_' {
                // Allow underscores
            } else {
                self.unknown = true;
            }
            return;
        }

        if self.starts_with_digit {
            if self.length == 2 && self.first_byte == b'0' && b == b'x' {
                self.in_hexadecimal = true;
                return;
            }

            if b == b'.' {
                if !self.has_decimal
                    && next
                        .map(|n| n.is_ascii_digit() || n == b'_')
                        .unwrap_or(true)
                {
                    self.has_decimal = true;
                } else {
                    self.unknown = true;
                }
            } else if b == b'e' || b == b'E' {
                if self.has_decimal && !self.has_exponent {
                    self.has_exponent = true;
                } else {
                    self.unknown = true;
                }
            } else if b.is_ascii_digit() {
                // Allow digits
            } else if b == b'_' {
                // Allow underscores
            } else {
                self.unknown = true;
            }
        }
    }
}

// https://tools.ietf.org/html/rfc3629
const UTF8_CHAR_WIDTH: &[u8; 256] = &[
    // 1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 1
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 2
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 3
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 4
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 5
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 6
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 7
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 8
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 9
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // A
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // B
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // C
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // D
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // E
    4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // F
];

/// Given a first byte, determines how many bytes are in this UTF-8 character.
#[inline(always)]
const fn utf8_char_width(b: u8) -> usize {
    UTF8_CHAR_WIDTH[b as usize] as usize
}
