#[cfg(test)]
mod tests;

use std::hint::cold_path;

use crate::{
    source::Span,
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
    utf8_validated: bool,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src [u8]) -> Self {
        Self {
            source,
            index: 0,
            token_start: None,
            token_flags: TokenFlags::default(),
            is_eof_or_error: false,
            utf8_validated: false,
        }
    }

    pub fn validated(source: &'src str) -> Self {
        Self {
            source: source.as_bytes(),
            index: 0,
            token_start: None,
            token_flags: TokenFlags::default(),
            is_eof_or_error: false,
            utf8_validated: true,
        }
    }

    pub fn source(&self) -> &'src [u8] {
        self.source
    }

    fn len(&self) -> usize {
        self.source.len()
    }

    fn current_byte(&self) -> Byte {
        if self.index < self.source.len() {
            Byte(self.source[self.index])
        } else {
            self.source.last().copied().map(Byte).unwrap_or_default()
        }
    }

    fn next_byte(&self) -> Option<Byte> {
        if self.index + 1 < self.source.len() {
            Some(Byte(self.source[self.index + 1]))
        } else {
            None
        }
    }

    fn finish_token(&mut self) -> Option<Token> {
        fn finish(kind: TokenKind, span: Span, lexer: &mut Lexer) -> Option<Token> {
            lexer.token_start = None;
            lexer.token_flags = TokenFlags::default();

            Some(Token { kind, span })
        }

        let start = self.token_start?;
        let span = Span::new(start, self.index);
        let bytes = &self.source[span.as_usize_range()];

        if self.token_flags.unknown || bytes.is_empty() {
            return finish(TokenKind::Unknown, span, self);
        }

        if self.token_flags.starts_with_digit {
            if self.token_flags.in_hexadecimal && self.token_flags.hex_digits > 0 {
                return finish(TokenKind::ByteValue, span, self);
            } else if self.token_flags.has_decimal {
                return finish(TokenKind::FloatValue, span, self);
            }

            return finish(TokenKind::IntegerValue, span, self);
        }

        let first_byte = Byte(bytes[0]);

        if first_byte
            .ascii_class()
            .is_some_and(|class| matches!(class, AsciiClass::ALPHABETICAL | AsciiClass::UNDERSCORE))
        {
            if let Some(keyword_kind) = keyword_kind(bytes) {
                return finish(keyword_kind, span, self);
            }

            return finish(TokenKind::Identifier, span, self);
        }

        if self.token_flags.unicode_identifier_started_non_ascii
            && self.token_flags.unicode_identifier_valid
        {
            return finish(TokenKind::Identifier, span, self);
        }

        finish(TokenKind::Unknown, span, self)
    }

    fn handle_non_ascii(&mut self) -> Result<(), usize> {
        match self.scan_utf8_sequence(self.index) {
            Ok(width) => {
                let first_byte = self.current_byte();
                let next_bytes = &self.source[self.index + 1..self.index + width];
                let code_point = decode_utf8_code_point(first_byte.0, next_bytes);

                if self.token_start.is_none() {
                    if is_xid_start(code_point) {
                        self.token_start = Some(self.index);
                        self.token_flags = TokenFlags::start(first_byte);
                        self.token_flags.saw_non_ascii = true;
                        self.token_flags.unicode_identifier_started_non_ascii = true;
                        self.token_flags.unicode_identifier_valid = true;
                    } else {
                        self.token_start = Some(self.index);
                        self.token_flags = TokenFlags::start(first_byte);
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

                Ok(())
            }
            Err(err_index) => {
                self.is_eof_or_error = true;

                Err(err_index)
            }
        }
    }

    fn scan_utf8_sequence(&self, start: usize) -> Result<usize, usize> {
        let first_byte = self.current_byte();

        if first_byte.is_ascii() {
            return Ok(1);
        }

        let width = first_byte.uft8_width();

        if width == 0 || start + width > self.source.len() {
            return Err(start);
        }

        if self.utf8_validated {
            return Ok(width);
        }

        match width {
            2 => {
                let second = self.source[start + 1];

                if (second as i8) >= -64 {
                    return Err(start);
                }
            }
            3 => {
                let second = self.source[start + 1];

                match (first_byte.0, second) {
                    (0xE0, 0xA0..=0xBF)
                    | (0xE1..=0xEC, 0x80..=0xBF)
                    | (0xED, 0x80..=0x9F)
                    | (0xEE..=0xEF, 0x80..=0xBF) => {}
                    _ => return Err(start),
                }

                let third = self.source[start + 2];

                if (third as i8) >= -64 {
                    return Err(start);
                }
            }
            4 => {
                let second = self.source[start + 1];

                match (first_byte.0, second) {
                    (0xF0, 0x90..=0xBF) | (0xF1..=0xF3, 0x80..=0xBF) | (0xF4, 0x80..=0x8F) => {}
                    _ => return Err(start),
                }

                let third = self.source[start + 2];

                if (third as i8) >= -64 {
                    return Err(start);
                }

                let fourth = self.source[start + 3];

                if (fourth as i8) >= -64 {
                    return Err(start);
                }
            }
            _ => return Err(start),
        }

        Ok(width)
    }

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

    fn classify_single_operator(&self) -> TokenKind {
        let byte = self.source[self.index];

        match byte {
            b'*' => TokenKind::Asterisk,
            b'!' => TokenKind::Bang,
            b'^' => TokenKind::Caret,
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
            _ => TokenKind::Unknown,
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token, usize>;

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

            let byte = self.current_byte();
            let Some(class) = byte.ascii_class() else {
                cold_path();

                match self.handle_non_ascii() {
                    Ok(()) => continue,
                    Err(err_index) => {
                        return Some(Err(err_index));
                    }
                }
            };

            // Skip whitespace
            if class == AsciiClass::WHITESPACE {
                if let Some(token) = self.finish_token() {
                    return Some(Ok(token));
                }

                self.index += 1;

                while self.index < self.len() {
                    let byte = self.current_byte();

                    if !byte.is_whitespace() {
                        break;
                    }

                    self.index += 1;
                }

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

            if class == AsciiClass::OPERATOR_OR_PUNCTUATION {
                if let Some(tok) = self.finish_token() {
                    return Some(Ok(tok));
                }

                if byte == b'-' && self.index + 9 <= self.len() {
                    let next = self.source[self.index + 1];

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
                    let operator_u16 =
                        u16::from_le_bytes([self.source[self.index], self.source[self.index + 1]]);

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
                let kind = self.classify_single_operator();
                self.index += 1;

                return Some(Ok(Token { kind, span }));
            }

            // Start a new token
            if self.token_start.is_none() {
                self.token_start = Some(self.index);
                self.token_flags = TokenFlags::start(byte);

                if !self.token_flags.starts_with_digit {
                    let mut next_index = self.index + 1;

                    while next_index < self.len() {
                        let next_byte = Byte(self.source[next_index]);

                        if next_byte.ascii_class().is_none_or(|class| {
                            matches!(
                                class,
                                AsciiClass::WHITESPACE | AsciiClass::OPERATOR_OR_PUNCTUATION
                            )
                        }) {
                            break;
                        }

                        next_index += 1;
                    }

                    self.index = next_index;

                    continue;
                }
            }

            if self.token_flags.starts_with_digit
                && let Some(start) = self.token_start
                && self.index > start
            {
                let next = self.next_byte();

                self.token_flags.push(byte, next);
            }

            self.index += 1;
        }
    }
}

fn keyword_kind(token: &[u8]) -> Option<TokenKind> {
    match (token.len(), token.first()?) {
        (2, b'a') if token == b"as" => Some(TokenKind::As),
        (2, b'f') if token == b"fn" => Some(TokenKind::Fn),
        (2, b'i') if token == b"if" => Some(TokenKind::If),

        (3, b'a') if token == b"any" => Some(TokenKind::Any),
        (3, b'i') if token == b"int" => Some(TokenKind::Int),
        (3, b'l') if token == b"let" => Some(TokenKind::Let),
        (3, b'm') => match token {
            b"map" => Some(TokenKind::Map),
            b"mod" => Some(TokenKind::Mod),
            b"mut" => Some(TokenKind::Mut),
            _ => None,
        },
        (3, b'p') if token == b"pub" => Some(TokenKind::Pub),
        (3, b's') if token == b"str" => Some(TokenKind::Str),
        (3, b'u') if token == b"use" => Some(TokenKind::Use),

        (4, b'b') => match token {
            b"bool" => Some(TokenKind::Bool),
            b"byte" => Some(TokenKind::Byte),
            _ => None,
        },
        (4, b'c') => match token {
            b"char" => Some(TokenKind::Char),
            b"cell" => Some(TokenKind::Cell),
            _ => None,
        },
        (4, b'e') if token == b"else" => Some(TokenKind::Else),
        (4, b'l') if token == b"loop" => Some(TokenKind::Loop),
        (4, b't') if token == b"true" => Some(TokenKind::TrueValue),

        (5, b'a') if token == b"async" => Some(TokenKind::Async),
        (5, b'b') if token == b"break" => Some(TokenKind::Break),
        (5, b'c') if token == b"const" => Some(TokenKind::Const),
        (5, b'f') => match token {
            b"false" => Some(TokenKind::FalseValue),
            b"float" => Some(TokenKind::Float),
            _ => None,
        },
        (5, b'w') if token == b"while" => Some(TokenKind::While),

        (6, b'r') if token == b"return" => Some(TokenKind::Return),
        (6, b's') if token == b"struct" => Some(TokenKind::Struct),

        (8, b'I') if token == b"Infinity" => Some(TokenKind::FloatValue),

        _ => None,
    }
}

fn classify_two_operator_u16(op: u16) -> Option<TokenKind> {
    Some(match op {
        0x3E2D => TokenKind::ArrowThin,
        0x3D2A => TokenKind::AsteriskEqual,
        0x3D21 => TokenKind::BangEqual,
        0x3D5E => TokenKind::CaretEqual,
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
    first_byte: Byte,
}

impl TokenFlags {
    fn start(first_byte: Byte) -> Self {
        Self {
            starts_with_digit: first_byte.is_ascii_digit(),
            in_hexadecimal: false,
            hex_digits: 0,
            has_decimal: false,
            has_exponent: false,
            unknown: false,
            saw_non_ascii: false,
            unicode_identifier_valid: true,
            unicode_identifier_started_non_ascii: false,
            len: 1,
            first_byte,
        }
    }

    fn push(&mut self, byte: Byte, next: Option<Byte>) {
        self.len += 1;

        if self.in_hexadecimal {
            if byte.is_ascii_hexdigit() {
                self.hex_digits += 1;

                return;
            }

            if byte == b'_' {
                return;
            }

            self.unknown = true;

            return;
        }

        if self.starts_with_digit {
            if self.len == 2 && self.first_byte == b'0' && byte == b'x' {
                self.in_hexadecimal = true;

                return;
            }

            if byte == b'.' {
                if self.has_decimal {
                    self.unknown = true;

                    return;
                }

                let next_is_digit = if let Some(next) = next
                    && let Some(class) = next.ascii_class()
                {
                    matches!(class, AsciiClass::DIGIT | AsciiClass::UNDERSCORE)
                } else {
                    false
                };

                if next_is_digit {
                    self.has_decimal = true;
                } else {
                    self.unknown = true;
                }

                return;
            }

            if byte == b'e' || byte == b'E' {
                if self.has_decimal && !self.has_exponent {
                    self.has_exponent = true;

                    return;
                } else {
                    self.unknown = true;

                    return;
                }
            }

            if byte
                .ascii_class()
                .is_none_or(|class| !matches!(class, AsciiClass::DIGIT | AsciiClass::UNDERSCORE))
            {
                self.unknown = true;
            }
        }
    }
}

fn decode_utf8_code_point(first: u8, tail: &[u8]) -> char {
    if first < 128 {
        return first as char;
    }

    if first & 0xE0 == 0xC0 {
        let bytes = ((first as u32 & 0x1F) << 6) | (tail[0] as u32 & 0x3F);

        return char::from_u32(bytes).unwrap();
    }

    if first & 0xF0 == 0xE0 {
        let bytes = ((first as u32 & 0x0F) << 12)
            | ((tail[0] as u32 & 0x3F) << 6)
            | (tail[1] as u32 & 0x3F);

        return char::from_u32(bytes).unwrap();
    }

    let bytes = ((first as u32 & 0x07) << 18)
        | ((tail[0] as u32 & 0x3F) << 12)
        | ((tail[1] as u32 & 0x3F) << 6)
        | (tail[2] as u32 & 0x3F);

    char::from_u32(bytes).unwrap()
}

// https://tools.ietf.org/html/rfc3629
const UTF8_CHAR_WIDTHS: &[u8; 256] = &[
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

#[derive(Debug, Clone, Copy, Default)]
struct Byte(u8);

impl Byte {
    fn ascii_class(&self) -> Option<AsciiClass> {
        if self.0.is_ascii() {
            Some(ASCII_CLASSES[self.0 as usize])
        } else {
            None
        }
    }

    fn is_ascii(&self) -> bool {
        self.0.is_ascii()
    }

    fn is_ascii_digit(&self) -> bool {
        self.0.is_ascii_digit()
    }

    fn is_ascii_hexdigit(&self) -> bool {
        self.0.is_ascii_hexdigit()
    }

    fn is_whitespace(&self) -> bool {
        self.0.is_ascii_whitespace()
    }

    fn uft8_width(&self) -> usize {
        UTF8_CHAR_WIDTHS[self.0 as usize] as usize
    }
}

impl PartialEq<u8> for Byte {
    fn eq(&self, other: &u8) -> bool {
        self.0 == *other
    }
}

#[derive(Clone, Copy, PartialEq)]
struct AsciiClass(u8);

impl AsciiClass {
    const CONTROL: Self = Self(0);
    const WHITESPACE: Self = Self(1);
    const OPERATOR_OR_PUNCTUATION: Self = Self(2);
    const DIGIT: Self = Self(4);
    const ALPHABETICAL: Self = Self(8);
    const UNDERSCORE: Self = Self(16);
}

const ASCII_CLASSES: [AsciiClass; 128] = {
    let mut classes = [AsciiClass(0); 128];
    let mut index = 0;

    while index < 128 {
        let character = index as u8 as char;

        match character {
            '0'..='9' => classes[index] = AsciiClass::DIGIT,
            'A'..='Z' | 'a'..='z' => classes[index] = AsciiClass::ALPHABETICAL,
            ' ' | '\t' | '\n' | '\r' => classes[index] = AsciiClass::WHITESPACE,
            '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-' | '.'
            | '/' | ':' | ';' | '<' | '=' | '>' | '?' | '@' | '[' | '\\' | ']' | '^' | '`'
            | '{' | '|' | '}' | '~' => classes[index] = AsciiClass::OPERATOR_OR_PUNCTUATION,
            '_' => classes[index] = AsciiClass::UNDERSCORE,
            '\0'..='\u{8}'
            | '\u{b}'
            | '\u{c}'
            | '\u{e}'..='\u{1f}'
            | '\u{7f}'..='\u{d7ff}'
            | '\u{e000}'..='\u{10ffff}' => classes[index] = AsciiClass::CONTROL,
        }

        index += 1;
    }

    classes
};
