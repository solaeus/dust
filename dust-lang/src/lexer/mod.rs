#[cfg(test)]
mod tests;

use std::hint::cold_path;

use crate::{
    Span,
    token::{Token, TokenKind},
};
use unicode_ident::{is_xid_continue, is_xid_start};

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
                let token = &self.source[start..end];

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
                } else if !token.is_empty() {
                    let first = token[0];

                    if first < 128 && (first.is_ascii_alphabetic() || first == b'_') {
                        if self.token_flags.saw_non_ascii {
                            if self.token_flags.unicode_identifier_valid {
                                TokenKind::Identifier
                            } else {
                                TokenKind::Unknown
                            }
                        } else if let Some(kind) = keyword_kind(token) {
                            kind
                        } else {
                            TokenKind::Identifier
                        }
                    } else if first >= 128 {
                        if self.token_flags.unicode_identifier_started_non_ascii
                            && self.token_flags.unicode_identifier_valid
                        {
                            TokenKind::Identifier
                        } else {
                            TokenKind::Unknown
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

            let byte = unsafe { *self.source.as_ptr().add(self.index) };

            // ASCII
            if byte < 0x80 {
                // Skip contiguous whitespace
                if is_ascii_whitespace(byte) {
                    if let Some(token) = self.finish_token() {
                        return Some(Ok(token));
                    }

                    let mut index = self.index + 1;
                    let end = self.len();
                    let source_poiner = self.source.as_ptr();

                    while index < end {
                        let byte = unsafe { *source_poiner.add(index) };

                        if byte >= 128 {
                            break;
                        }

                        if (ASCII_CLASS[byte as usize] & CLASS_WHITESPACE) == 0 {
                            break;
                        }

                        index += 1;
                    }

                    self.index = index;

                    continue;
                }

                // String literal
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

                // Character literal
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

                // Float literal
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
                        self.token_flags.len += 1;
                        self.token_flags.has_decimal = true;

                        continue;
                    }
                }

                if is_operator_or_punctuation(byte) {
                    if let Some(tok) = self.finish_token() {
                        return Some(Ok(tok));
                    }

                    if byte == b'-' && self.index + 9 <= self.len() {
                        let next = unsafe { *self.source.as_ptr().add(self.index + 1) };

                        if next == b'I' {
                            let slice = &self.source[self.index..self.index + 9];

                            if slice == b"-Infinity" {
                                let span = Span(self.index as u32, (self.index + 9) as u32);
                                let kind = TokenKind::FloatValue;
                                self.index += 9;

                                return Some(Ok(Token { kind, span }));
                            }
                        }
                    }

                    if self.index + 1 < self.len() {
                        let operator_u16 = unsafe {
                            u16::from_le_bytes([
                                *self.source.as_ptr().add(self.index),
                                *self.source.as_ptr().add(self.index + 1),
                            ])
                        };

                        if let Some(two_kind) = classify_two_operator_u16(operator_u16) {
                            let span = Span(self.index as u32, (self.index + 2) as u32);

                            self.index += 2;

                            return Some(Ok(Token {
                                kind: two_kind,
                                span,
                            }));
                        }
                    }

                    let span = Span(self.index as u32, (self.index + 1) as u32);
                    let kind =
                        classify_single_operator(unsafe { *self.source.as_ptr().add(self.index) });
                    self.index += 1;

                    return Some(Ok(Token { kind, span }));
                }

                if self.token_start.is_none() {
                    self.token_start = Some(self.index);
                    self.token_flags = TokenFlags::start(byte);

                    if self.token_flags.starts_with_digit {
                        let next = unsafe {
                            if self.index + 1 < self.len() {
                                Some(*self.source.as_ptr().add(self.index + 1))
                            } else {
                                None
                            }
                        };

                        self.token_flags.push(byte, next);
                    } else {
                        let mut index = self.index + 1;
                        let end = self.len();
                        let pointer = self.source.as_ptr();

                        while index < end {
                            let byte = unsafe { *pointer.add(index) };

                            if byte >= 128 {
                                break;
                            }

                            let class = ASCII_CLASS[byte as usize];

                            if (class & (CLASS_WHITESPACE | CLASS_PUNCTUATION)) != 0 {
                                break;
                            }

                            index += 1;
                        }

                        self.index = index;

                        continue;
                    }
                }

                self.index += 1;

                continue;
            }

            cold_path();

            match self.scan_utf8_sequence(self.index) {
                Ok(width) => {
                    let first = unsafe { *self.source.as_ptr().add(self.index) };
                    let next_slice_start = self.index + 1;
                    let code_point = decode_utf8_code_point(
                        first,
                        &self.source[next_slice_start..next_slice_start + (width - 1)],
                    );

                    if self.token_start.is_none() {
                        if is_xid_start(code_point) {
                            self.token_start = Some(self.index);
                            self.token_flags = TokenFlags::start(first);
                            self.token_flags.saw_non_ascii = true;
                            self.token_flags.unicode_identifier_started_non_ascii = true;
                            self.token_flags.unicode_identifier_valid = true;
                        } else {
                            self.token_start = Some(self.index);
                            self.token_flags = TokenFlags::start(first);
                            self.token_flags.saw_non_ascii = true;
                            self.token_flags.unknown = true;
                        }
                    } else {
                        self.token_flags.saw_non_ascii = true;

                        if self.token_flags.starts_with_digit {
                            self.token_flags.unknown = true;
                        } else {
                            let is_valid_continue = is_xid_continue(code_point);
                            self.token_flags.unicode_identifier_valid =
                                self.token_flags.unicode_identifier_valid && is_valid_continue;
                        }
                    }

                    self.token_flags.len = self.token_flags.len.saturating_add(width);
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

#[inline(always)]
fn is_ascii_whitespace(byte: u8) -> bool {
    byte < 128 && (ASCII_CLASS[byte as usize] & CLASS_WHITESPACE) != 0
}

#[inline(always)]
fn is_operator_or_punctuation(byte: u8) -> bool {
    byte < 128 && (ASCII_CLASS[byte as usize] & CLASS_PUNCTUATION) != 0
}

#[inline(always)]
fn keyword_kind(token: &[u8]) -> Option<TokenKind> {
    match token.len() {
        2 => match token[0] {
            b'f' => {
                if token[1] == b'n' {
                    Some(TokenKind::Fn)
                } else {
                    None
                }
            }
            b'i' => {
                if token[1] == b'f' {
                    Some(TokenKind::If)
                } else {
                    None
                }
            }
            _ => None,
        },
        3 => match token[0] {
            b'a' => {
                if &token[1..3] == b"ny" {
                    Some(TokenKind::Any)
                } else {
                    None
                }
            }
            b'i' => {
                if &token[1..3] == b"nt" {
                    Some(TokenKind::Int)
                } else {
                    None
                }
            }
            b'l' => {
                if &token[1..3] == b"et" {
                    Some(TokenKind::Let)
                } else {
                    None
                }
            }
            b'm' => match &token[1..3] {
                b"ap" => Some(TokenKind::Map),
                b"od" => Some(TokenKind::Mod),
                b"ut" => Some(TokenKind::Mut),
                _ => None,
            },
            b'p' => {
                if &token[1..3] == b"ub" {
                    Some(TokenKind::Pub)
                } else {
                    None
                }
            }
            b's' => {
                if &token[1..3] == b"tr" {
                    Some(TokenKind::Str)
                } else {
                    None
                }
            }
            b'u' => {
                if &token[1..3] == b"se" {
                    Some(TokenKind::Use)
                } else {
                    None
                }
            }
            _ => None,
        },
        4 => match token[0] {
            b'b' => match &token[1..4] {
                b"ool" => Some(TokenKind::Bool),
                b"yte" => Some(TokenKind::Byte),
                _ => None,
            },
            b'c' => match &token[1..4] {
                b"har" => Some(TokenKind::Char),
                b"ell" => Some(TokenKind::Cell),
                _ => None,
            },
            b'e' => {
                if &token[1..4] == b"lse" {
                    Some(TokenKind::Else)
                } else {
                    None
                }
            }
            b'l' => match &token[1..4] {
                b"ist" => Some(TokenKind::List),
                b"oop" => Some(TokenKind::Loop),
                _ => None,
            },
            b't' => {
                if &token[1..4] == b"rue" {
                    Some(TokenKind::TrueValue)
                } else {
                    None
                }
            }
            _ => None,
        },
        5 => match token[0] {
            b'a' => {
                if &token[1..5] == b"sync" {
                    Some(TokenKind::Async)
                } else {
                    None
                }
            }
            b'b' => {
                if &token[1..5] == b"reak" {
                    Some(TokenKind::Break)
                } else {
                    None
                }
            }
            b'c' => {
                if &token[1..5] == b"onst" {
                    Some(TokenKind::Const)
                } else {
                    None
                }
            }
            b'f' => match &token[1..5] {
                b"alse" => Some(TokenKind::FalseValue),
                b"loat" => Some(TokenKind::Float),
                _ => None,
            },
            b'w' => {
                if &token[1..5] == b"hile" {
                    Some(TokenKind::While)
                } else {
                    None
                }
            }
            _ => None,
        },
        6 => match token[0] {
            b'r' => {
                if &token[1..6] == b"eturn" {
                    Some(TokenKind::Return)
                } else {
                    None
                }
            }
            b's' => {
                if &token[1..6] == b"truct" {
                    Some(TokenKind::Struct)
                } else {
                    None
                }
            }
            _ => None,
        },
        8 => {
            if token == b"Infinity" {
                Some(TokenKind::FloatValue)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[inline(always)]
fn classify_two_operator_u16(op: u16) -> Option<TokenKind> {
    Some(match op {
        0x3E2D => TokenKind::ArrowThin,
        0x3D2A => TokenKind::AsteriskEqual,
        0x3D21 => TokenKind::BangEqual,
        0x2626 => TokenKind::DoubleAmpersand,
        0x3A3A => TokenKind::DoubleColon,
        0x2E2E => TokenKind::DoubleDot,
        0x3D3D => TokenKind::DoubleEqual,
        0x7C7C => TokenKind::DoublePipe,
        0x3D3E => TokenKind::GreaterEqual,
        0x3D3C => TokenKind::LessEqual,
        0x3D2D => TokenKind::MinusEqual,
        0x3D25 => TokenKind::PercentEqual,
        0x3D2B => TokenKind::PlusEqual,
        0x3D2F => TokenKind::SlashEqual,
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
    unicode_identifier_valid: bool,
    unicode_identifier_started_non_ascii: bool,
    len: usize,
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
            unicode_identifier_valid: true,
            unicode_identifier_started_non_ascii: false,
            len: 1,
            first_byte: first,
        }
    }

    #[inline(always)]
    fn push(&mut self, b: u8, next: Option<u8>) {
        self.len += 1;

        if self.in_hexadecimal {
            if b.is_ascii_hexdigit() {
                self.hex_digits += 1;
                return;
            }
            if b == b'_' {
                return;
            }
            self.unknown = true;
            return;
        }

        if self.starts_with_digit {
            if self.len == 2 && self.first_byte == b'0' && b == b'x' {
                self.in_hexadecimal = true;
                return;
            }

            let class = if b < 128 { ASCII_CLASS[b as usize] } else { 0 };

            if b == b'.' {
                if !self.has_decimal
                    && next
                        .map(|n| {
                            if n < 128 {
                                let c = ASCII_CLASS[n as usize];
                                (c & CLASS_DIGIT) != 0 || (c & CLASS_UNDERSCORE) != 0
                            } else {
                                false
                            }
                        })
                        .unwrap_or(true)
                {
                    self.has_decimal = true;
                    return;
                } else {
                    self.unknown = true;
                    return;
                }
            }

            if b == b'e' || b == b'E' {
                if self.has_decimal && !self.has_exponent {
                    self.has_exponent = true;
                    return;
                } else {
                    self.unknown = true;
                    return;
                }
            }

            if (class & CLASS_DIGIT) != 0 || (class & CLASS_UNDERSCORE) != 0 {
                return;
            }

            self.unknown = true;
        }
    }
}

/// Given a first byte, determines how many bytes are in this UTF-8 character.
#[inline(always)]
const fn utf8_char_width(b: u8) -> usize {
    UTF8_CHAR_WIDTH[b as usize] as usize
}

#[inline(always)]
fn decode_utf8_code_point(first: u8, tail: &[u8]) -> char {
    if first < 0x80 {
        return first as char;
    }
    if first & 0xE0 == 0xC0 {
        let u = ((first as u32 & 0x1F) << 6) | (tail[0] as u32 & 0x3F);
        return unsafe { char::from_u32_unchecked(u) };
    }
    if first & 0xF0 == 0xE0 {
        let u = ((first as u32 & 0x0F) << 12)
            | ((tail[0] as u32 & 0x3F) << 6)
            | (tail[1] as u32 & 0x3F);
        return unsafe { char::from_u32_unchecked(u) };
    }
    let u = ((first as u32 & 0x07) << 18)
        | ((tail[0] as u32 & 0x3F) << 12)
        | ((tail[1] as u32 & 0x3F) << 6)
        | (tail[2] as u32 & 0x3F);
    unsafe { char::from_u32_unchecked(u) }
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

const CLASS_WHITESPACE: u8 = 1;
const CLASS_PUNCTUATION: u8 = 2;
const CLASS_DIGIT: u8 = 4;
const CLASS_UNDERSCORE: u8 = 16;

const ASCII_CLASS: [u8; 128] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 2, 2, 2, 2, 2,
    2, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 2, 2, 2, 2,
    16, 2, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 2, 2, 2,
    2, 0,
];
