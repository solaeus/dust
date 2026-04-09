#[cfg(test)]
mod tests;

use std::hint::cold_path;

use crate::{
    error::ErrorKind,
    parser::error::ParseError,
    source::{Code, Position, Source, Span},
    token::{Token, TokenKind},
};
use unicode_ident::{is_xid_continue, is_xid_start};

pub fn tokenize_bytes(bytes: &[u8]) -> Result<Vec<Token>, ErrorKind> {
    let mut source = Source::with_capacity(1);
    let file_id = source.add_code(Code::borrowed("tokenize", bytes));

    let mut lexer = Lexer::with_unvalidated_source(bytes);
    let mut tokens = Vec::new();

    for token in &mut lexer {
        tokens.push(token);
    }

    if lexer.error {
        let error_index = lexer.error_index().unwrap_or(0);
        let position = Position::new(file_id, Span::new(error_index, error_index));

        return Err(ErrorKind::Parse(ParseError::InvalidUtf8 { position }));
    }

    Ok(tokens)
}

pub fn tokenize_str(str: &str) -> Result<Vec<Token>, ErrorKind> {
    let mut source = Source::with_capacity(1);
    let file_id = source.add_code(Code::validated_borrowed("tokenize", str));

    let mut lexer = Lexer::with_validated_source(str);
    let mut tokens = Vec::new();

    for token in &mut lexer {
        tokens.push(token);
    }

    if let Some(error_index) = lexer.error_index() {
        let position = Position::new(file_id, Span::new(error_index, error_index));

        return Err(ErrorKind::Parse(ParseError::InvalidUtf8 { position }));
    }

    Ok(tokens)
}

#[derive(Debug)]
pub struct Lexer<'src> {
    source: &'src [u8],
    index: usize,
    token_start: Option<usize>,
    token_flags: TokenFlags,
    eof: bool,
    error: bool,
    utf8_validated: bool,
}

impl<'src> Lexer<'src> {
    pub fn with_unvalidated_source(source: &'src [u8]) -> Self {
        Self {
            source,
            index: 0,
            token_start: None,
            token_flags: TokenFlags::default(),
            eof: false,
            error: false,
            utf8_validated: false,
        }
    }

    pub fn with_validated_source(source: &'src str) -> Self {
        Self {
            source: source.as_bytes(),
            index: 0,
            token_start: None,
            token_flags: TokenFlags::default(),
            eof: false,
            error: false,
            utf8_validated: true,
        }
    }

    pub fn source(&self) -> &'src [u8] {
        self.source
    }

    pub fn error_index(&self) -> Option<usize> {
        if self.error { Some(self.index) } else { None }
    }

    #[inline(always)]
    fn next_byte(&self) -> Option<u8> {
        self.source.get(self.index + 1).copied()
    }

    #[inline(always)]
    fn operator_or_punctuation(&self, current: u8) -> (TokenKind, usize) {
        let next = self.next_byte();

        match current {
            b'*' => {
                if let Some(next) = next
                    && next == b'='
                {
                    (TokenKind::AsteriskEqual, 2)
                } else {
                    (TokenKind::Asterisk, 1)
                }
            }
            b'!' => {
                if let Some(next) = next
                    && next == b'='
                {
                    (TokenKind::BangEqual, 2)
                } else {
                    (TokenKind::Bang, 1)
                }
            }
            b'^' => {
                if let Some(next) = next
                    && next == b'='
                {
                    (TokenKind::CaretEqual, 2)
                } else {
                    (TokenKind::Caret, 1)
                }
            }
            b':' => {
                if let Some(next) = next
                    && next == b':'
                {
                    (TokenKind::DoubleColon, 2)
                } else {
                    (TokenKind::Colon, 1)
                }
            }
            b',' => (TokenKind::Comma, 1),
            b'.' => {
                if let Some(next) = next
                    && next == b'.'
                {
                    if self.source.get(self.index + 2) == Some(&b'=') {
                        (TokenKind::DoubleDotEqual, 3)
                    } else {
                        (TokenKind::DoubleDot, 2)
                    }
                } else {
                    (TokenKind::Dot, 1)
                }
            }
            b'=' => {
                if let Some(next) = next
                    && next == b'='
                {
                    (TokenKind::DoubleEqual, 2)
                } else {
                    (TokenKind::Equal, 1)
                }
            }
            b'>' => {
                if let Some(next) = next
                    && next == b'='
                {
                    (TokenKind::GreaterEqual, 2)
                } else {
                    (TokenKind::Greater, 1)
                }
            }
            b'{' => (TokenKind::LeftCurlyBrace, 1),
            b'[' => (TokenKind::LeftSquareBracket, 1),
            b'(' => (TokenKind::LeftParenthesis, 1),
            b'<' => {
                if let Some(next) = next
                    && next == b'='
                {
                    (TokenKind::LessEqual, 2)
                } else {
                    (TokenKind::Less, 1)
                }
            }
            b'-' => {
                if let Some(next) = next {
                    match next {
                        b'=' => (TokenKind::MinusEqual, 2),
                        b'>' => (TokenKind::ArrowThin, 2),
                        _ => (TokenKind::Minus, 1),
                    }
                } else {
                    (TokenKind::Minus, 1)
                }
            }
            b'%' => {
                if let Some(next) = next
                    && next == b'='
                {
                    (TokenKind::PercentEqual, 2)
                } else {
                    (TokenKind::Percent, 1)
                }
            }
            b'+' => {
                if let Some(next) = next
                    && next == b'='
                {
                    (TokenKind::PlusEqual, 2)
                } else {
                    (TokenKind::Plus, 1)
                }
            }
            b'}' => (TokenKind::RightCurlyBrace, 1),
            b']' => (TokenKind::RightSquareBracket, 1),
            b')' => (TokenKind::RightParenthesis, 1),
            b';' => (TokenKind::Semicolon, 1),
            b'/' => {
                if let Some(next) = next
                    && next == b'='
                {
                    (TokenKind::SlashEqual, 2)
                } else {
                    (TokenKind::Slash, 1)
                }
            }
            _ => {
                let Some(next) = next else {
                    return (TokenKind::Unknown, 1);
                };

                match (current, next) {
                    (b'&', b'&') => (TokenKind::DoubleAmpersand, 2),
                    (b'|', b'|') => (TokenKind::DoublePipe, 2),
                    _ => (TokenKind::Unknown, 1),
                }
            }
        }
    }

    #[inline(always)]
    fn finish_token(&mut self) -> Option<Token> {
        #[inline(always)]
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
                return finish(TokenKind::HexIntegerLiteral, span, self);
            } else if self.token_flags.has_decimal {
                return finish(TokenKind::FloatLiteral, span, self);
            }

            return finish(TokenKind::IntegerLiteral, span, self);
        }

        let class = bytes[0].class();

        if class.is_alphabetical() || class.is_underscore() {
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

    #[inline(always)]
    fn handle_non_ascii(&mut self) -> Result<Option<Token>, ()> {
        match self.scan_utf8_sequence(self.index) {
            Ok(width) => {
                let first_byte = self.source[self.index];
                let next_bytes = &self.source[self.index + 1..self.index + width];
                let code_point = decode_utf8_code_point(first_byte, next_bytes);

                if self.token_start.is_none() {
                    if is_xid_start(code_point) {
                        self.token_start = Some(self.index);
                        self.token_flags = TokenFlags::new(first_byte);
                        self.token_flags.saw_non_ascii = true;
                        self.token_flags.unicode_identifier_started_non_ascii = true;
                        self.token_flags.unicode_identifier_valid = true;
                    } else {
                        self.token_start = Some(self.index);
                        self.token_flags = TokenFlags::new(first_byte);
                        self.token_flags.saw_non_ascii = true;
                        self.token_flags.unknown = true;
                    }
                } else {
                    self.token_flags.saw_non_ascii = true;

                    if self.token_flags.starts_with_digit {
                        self.token_flags.unknown = true;
                    } else {
                        let is_valid_continue = is_xid_continue(code_point);

                        if !is_valid_continue
                            && !self.token_flags.unicode_identifier_started_non_ascii
                        {
                            return Ok(self.finish_token());
                        }

                        self.token_flags.unicode_identifier_valid =
                            self.token_flags.unicode_identifier_valid && is_valid_continue;
                    }
                }

                self.token_flags.len = self.token_flags.len.saturating_add(width);
                self.index += width;

                if self.token_flags.unknown && self.index < self.source.len() {
                    let next_class = self.source[self.index].class();

                    if next_class.is_alphabetical() || next_class.is_underscore() {
                        return Ok(self.finish_token());
                    }
                }

                Ok(None)
            }
            Err(()) => Err(()),
        }
    }

    #[inline(always)]
    fn scan_utf8_sequence(&mut self, start: usize) -> Result<usize, ()> {
        let first_byte = self.source[start];

        if first_byte.is_ascii() {
            return Ok(1);
        }

        let width = first_byte.utf8_width();

        if width == 0 || start + width > self.source.len() {
            {
                self.error = true;
                self.index = start;

                return Err(());
            }
        }

        if self.utf8_validated {
            return Ok(width);
        }

        match width {
            2 => {
                let second = self.source[start + 1];

                if (second as i8) >= -64 {
                    {
                        self.error = true;
                        self.index = start;

                        return Err(());
                    }
                }
            }
            3 => {
                let second = self.source[start + 1];

                match (first_byte, second) {
                    (0xE0, 0xA0..=0xBF)
                    | (0xE1..=0xEC, 0x80..=0xBF)
                    | (0xED, 0x80..=0x9F)
                    | (0xEE..=0xEF, 0x80..=0xBF) => {}
                    _ => {
                        self.error = true;
                        self.index = start;

                        return Err(());
                    }
                }

                let third = self.source[start + 2];

                if (third as i8) >= -64 {
                    {
                        self.error = true;
                        self.index = start;

                        return Err(());
                    }
                }
            }
            4 => {
                let second = self.source[start + 1];

                match (first_byte, second) {
                    (0xF0, 0x90..=0xBF) | (0xF1..=0xF3, 0x80..=0xBF) | (0xF4, 0x80..=0x8F) => {}
                    _ => {
                        self.error = true;
                        self.index = start;

                        return Err(());
                    }
                }

                let third = self.source[start + 2];

                if (third as i8) >= -64 {
                    {
                        self.error = true;
                        self.index = start;

                        return Err(());
                    }
                }

                let fourth = self.source[start + 3];

                if (fourth as i8) >= -64 {
                    {
                        self.error = true;
                        self.index = start;

                        return Err(());
                    }
                }
            }
            _ => {
                self.error = true;
                self.index = start;

                return Err(());
            }
        }

        Ok(width)
    }

    #[inline(always)]
    fn scan_string(&mut self) -> Result<Option<Token>, ()> {
        let start = self.index;

        if self.source[start] != b'"' {
            return Ok(None);
        }

        let mut index = start + 1;

        while index < self.source.len() {
            let byte = self.source[index];

            if byte == b'"' {
                self.index = index + 1;

                return Ok(Some(Token {
                    kind: TokenKind::StringLiteral,
                    span: Span::new(start, self.index),
                }));
            }

            if byte.is_ascii() {
                index += 1;
            } else {
                match self.scan_utf8_sequence(index) {
                    Ok(width) => index += width,
                    Err(()) => return Err(()),
                }
            }
        }

        let unknown_span = Span::new(start, self.index);

        self.index = self.source.len();

        Ok(Some(Token {
            kind: TokenKind::Unknown,
            span: unknown_span,
        }))
    }

    #[inline(always)]
    fn scan_chararacter(&mut self) -> Result<Option<Token>, ()> {
        let start = self.index;

        if self.source[start] != b'\'' {
            return Ok(None);
        }

        let mut index = start + 1;

        while index < self.source.len() {
            let byte = self.source[index];

            if byte < 0x80 {
                if byte == b'\'' {
                    let end = index + 1;

                    self.index = end;

                    return Ok(Some(Token {
                        kind: TokenKind::CharacterLiteral,
                        span: Span::new(start, end),
                    }));
                } else if byte == b'\\' {
                    index += 2;
                } else {
                    index += 1;
                }
            } else {
                match self.scan_utf8_sequence(index) {
                    Ok(width) => index += width,
                    Err(()) => return Err(()),
                }
            }
        }

        let end = self.source.len();

        self.index = end;

        Ok(Some(Token {
            kind: TokenKind::CharacterLiteral,
            span: Span::new(start, end),
        }))
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.eof || self.error {
                return None;
            }

            if self.index >= self.source.len() {
                if let Some(token) = self.finish_token() {
                    return Some(token);
                }

                self.eof = true;

                return Some(Token {
                    kind: TokenKind::Eof,
                    span: Span::new(self.source.len(), self.source.len()),
                });
            }

            let current_byte = self.source[self.index];

            if current_byte.is_ascii_whitespace() {
                if let Some(token) = self.finish_token() {
                    return Some(token);
                }

                self.index += 1;

                while self.index < self.source.len() {
                    let current_byte = self.source[self.index];

                    if !current_byte.is_ascii_whitespace() {
                        break;
                    }

                    self.index += 1;
                }

                if self.index >= self.source.len() {
                    if let Some(token) = self.finish_token() {
                        return Some(token);
                    }

                    self.eof = true;

                    return Some(Token {
                        kind: TokenKind::Eof,
                        span: Span::new(self.source.len(), self.source.len()),
                    });
                }
            }

            let current_byte = self.source[self.index];

            if current_byte == b'"' {
                if let Some(token) = self.finish_token() {
                    return Some(token);
                }

                match self.scan_string() {
                    Ok(Some(token)) => return Some(token),
                    Ok(None) => {
                        self.index += 1;

                        continue;
                    }
                    Err(()) => return None,
                }
            }

            if current_byte == b'\'' {
                if let Some(token) = self.finish_token() {
                    return Some(token);
                }

                match self.scan_chararacter() {
                    Ok(Some(token)) => return Some(token),
                    Ok(None) => {
                        self.index += 1;

                        continue;
                    }
                    Err(()) => return None,
                }
            }

            if current_byte == b'.'
                && let Some(start) = self.token_start
            {
                if self.token_flags.has_decimal
                    && let Some(token) = self.finish_token()
                {
                    return Some(token);
                }

                let first_byte = self.source[start];

                let next_is_digit = (self.index + 1) < self.source.len() && {
                    let byte = self.source[self.index + 1];

                    byte.is_ascii_digit() || byte == b'_'
                };

                if first_byte.is_ascii_digit() && next_is_digit {
                    self.index += 1;
                    self.token_flags.len += 1;
                    self.token_flags.has_decimal = true;

                    continue;
                }
            }

            let current_class = current_byte.class();

            if current_class.is_operator_or_punctuation() {
                if let Some(token) = self.finish_token() {
                    return Some(token);
                }

                if current_byte == b'-' && self.index + 9 <= self.source.len() {
                    let next_byte = self.source[self.index + 1];

                    if next_byte == b'I' {
                        let slice = &self.source[self.index..self.index + 9];

                        if slice == b"-Infinity" {
                            let span = Span::new(self.index, self.index + 9);
                            let kind = TokenKind::FloatLiteral;
                            self.index += 9;

                            return Some(Token { kind, span });
                        }
                    }
                }

                let (kind, width) = self.operator_or_punctuation(current_byte);

                let span = Span::new(self.index, self.index + width);
                self.index += width;

                return Some(Token { kind, span });
            }

            if self.token_start.is_none() && current_class.is_ascii() {
                self.token_start = Some(self.index);
                self.token_flags = TokenFlags::new(current_byte);

                if !self.token_flags.starts_with_digit {
                    let mut next_index = self.index + 1;

                    while next_index < self.source.len() {
                        let next_class = self.source[next_index].class();

                        if next_class.is_whitespace()
                            || next_class.is_operator_or_punctuation()
                            || !next_class.is_ascii()
                        {
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
                let next_byte = self.next_byte();

                self.token_flags.push(current_byte, next_byte);
            }

            if !current_class.is_ascii() {
                cold_path();

                match self.handle_non_ascii() {
                    Ok(Some(token)) => return Some(token),
                    Ok(None) => continue,
                    Err(_) => return None,
                }
            }

            self.index += 1;
        }
    }
}

impl ExactSizeIterator for Lexer<'_> {
    fn len(&self) -> usize {
        self.source.len() - self.index
    }
}

#[inline(always)]
fn keyword_kind(token: &[u8]) -> Option<TokenKind> {
    match token.len() {
        2 => match token[0] {
            b'a' => {
                if token[1] == b's' {
                    Some(TokenKind::As)
                } else {
                    None
                }
            }
            b'f' => {
                if token[1] == b'n' {
                    Some(TokenKind::Fn)
                } else {
                    None
                }
            }
            b'i' => match token[1] {
                b'f' => Some(TokenKind::If),
                b'8' => Some(TokenKind::I8),
                _ => None,
            },
            b'u' => match token[1] {
                b'8' => Some(TokenKind::U8),
                _ => None,
            },
            _ => None,
        },
        3 => match token[0] {
            b'f' => match &token[1..3] {
                b"32" => Some(TokenKind::F32),
                b"64" => Some(TokenKind::F64),
                b"or" => Some(TokenKind::For),
                _ => None,
            },
            b'i' => match &token[1..3] {
                b"16" => Some(TokenKind::I16),
                b"32" => Some(TokenKind::I32),
                b"64" => Some(TokenKind::I64),
                _ => None,
            },
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
            b'u' => match &token[1..3] {
                b"16" => Some(TokenKind::U16),
                b"32" => Some(TokenKind::U32),
                b"64" => Some(TokenKind::U64),
                b"se" => Some(TokenKind::Use),
                _ => None,
            },
            _ => None,
        },
        4 => match token[0] {
            b'S' => match &token[1..4] {
                b"elf" => Some(TokenKind::SelfType),
                _ => None,
            },
            b'b' => match &token[1..4] {
                b"ool" => Some(TokenKind::Bool),
                _ => None,
            },
            b'c' => match &token[1..4] {
                b"har" => Some(TokenKind::Char),
                b"ell" => Some(TokenKind::Cell),
                _ => None,
            },
            b'e' => match &token[1..4] {
                b"lse" => Some(TokenKind::Else),
                b"num" => Some(TokenKind::Enum),
                _ => None,
            },
            b'i' => match &token[1..4] {
                b"128" => Some(TokenKind::I128),
                b"mpl" => Some(TokenKind::Impl),
                _ => None,
            },
            b'l' => match &token[1..4] {
                b"oop" => Some(TokenKind::Loop),
                _ => None,
            },
            b's' => match &token[1..4] {
                b"elf" => Some(TokenKind::SelfValue),
                _ => None,
            },
            b't' => match &token[1..4] {
                b"rue" => Some(TokenKind::True),
                b"ype" => Some(TokenKind::Type),
                _ => None,
            },
            b'u' => match &token[1..4] {
                b"128" => Some(TokenKind::U128),
                _ => None,
            },
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
                b"alse" => Some(TokenKind::False),
                _ => None,
            },
            b'i' => {
                if &token[1..5] == b"size" {
                    Some(TokenKind::ISize)
                } else {
                    None
                }
            }
            b't' => {
                if &token[1..5] == b"rait" {
                    Some(TokenKind::Trait)
                } else {
                    None
                }
            }
            b'u' => {
                if &token[1..5] == b"size" {
                    Some(TokenKind::USize)
                } else {
                    None
                }
            }
            b'w' => match &token[1..5] {
                b"here" => Some(TokenKind::Where),
                b"hile" => Some(TokenKind::While),
                _ => None,
            },
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
                Some(TokenKind::FloatLiteral)
            } else {
                None
            }
        }
        _ => None,
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
    fn new(first_byte: u8) -> Self {
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

    #[inline(always)]
    fn push(&mut self, byte: u8, next: Option<u8>) {
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

                let Some(next) = next else {
                    self.unknown = true;

                    return;
                };
                let next_class = next.class();

                if next_class.is_digit() || next_class.is_underscore() {
                    self.has_decimal = true;
                } else if !next_class.is_ascii() {
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

            let class = byte.class();

            if !class.is_digit() && !class.is_underscore() && class.is_ascii() {
                self.unknown = true;
            }
        }
    }
}

#[inline(always)]
fn decode_utf8_code_point(first: u8, tail: &[u8]) -> char {
    if first < 128 {
        return first as char;
    }

    if first & 0xE0 == 0xC0 {
        let bytes = ((first as u32 & 0x1F) << 6) | (tail[0] as u32 & 0x3F);

        return char::from_u32(bytes).unwrap_or_default();
    }

    if first & 0xF0 == 0xE0 {
        let bytes = ((first as u32 & 0x0F) << 12)
            | ((tail[0] as u32 & 0x3F) << 6)
            | (tail[1] as u32 & 0x3F);

        return char::from_u32(bytes).unwrap_or_default();
    }

    let bytes = ((first as u32 & 0x07) << 18)
        | ((tail[0] as u32 & 0x3F) << 12)
        | ((tail[1] as u32 & 0x3F) << 6)
        | (tail[2] as u32 & 0x3F);

    char::from_u32(bytes).unwrap_or_default()
}

#[derive(Clone, Copy)]
struct Utf8Class(u8);

impl Utf8Class {
    const CONTROL: Self = Self(0);
    const WHITESPACE: Self = Self(1);
    const OPERATOR_OR_PUNCTUATION: Self = Self(2);
    const DIGIT: Self = Self(4);
    const ALPHABETICAL: Self = Self(8);
    const UNDERSCORE: Self = Self(16);
    const NON_ASCII: Self = Self(32);

    #[inline(always)]
    fn is_ascii(&self) -> bool {
        (self.0 & Self::NON_ASCII.0) == 0
    }

    #[inline(always)]
    fn is_whitespace(&self) -> bool {
        (self.0 & Self::WHITESPACE.0) != 0
    }

    #[inline(always)]
    fn is_operator_or_punctuation(&self) -> bool {
        (self.0 & Self::OPERATOR_OR_PUNCTUATION.0) != 0
    }

    #[inline(always)]
    fn is_digit(&self) -> bool {
        (self.0 & Self::DIGIT.0) != 0
    }

    #[inline(always)]
    fn is_alphabetical(&self) -> bool {
        (self.0 & Self::ALPHABETICAL.0) != 0
    }

    #[inline(always)]
    fn is_underscore(&self) -> bool {
        (self.0 & Self::UNDERSCORE.0) != 0
    }
}

const ASCII_CLASSES: [Utf8Class; 128] = {
    let mut classes = [Utf8Class(0); 128];
    let mut index = 0;

    while index < 128 {
        let character = index as u8 as char;

        match character {
            '0'..='9' => classes[index] = Utf8Class::DIGIT,
            'A'..='Z' | 'a'..='z' => classes[index] = Utf8Class::ALPHABETICAL,
            ' ' | '\t' | '\n' | '\r' => classes[index] = Utf8Class::WHITESPACE,
            '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-' | '.'
            | '/' | ':' | ';' | '<' | '=' | '>' | '?' | '@' | '[' | '\\' | ']' | '^' | '`'
            | '{' | '|' | '}' | '~' => classes[index] = Utf8Class::OPERATOR_OR_PUNCTUATION,
            '_' => classes[index] = Utf8Class::UNDERSCORE,
            '\0'..='\u{8}'
            | '\u{b}'
            | '\u{c}'
            | '\u{e}'..='\u{1f}'
            | '\u{7f}'..='\u{d7ff}'
            | '\u{e000}'..='\u{10ffff}' => classes[index] = Utf8Class::CONTROL,
        }

        index += 1;
    }

    classes
};

trait Utf8Byte {
    fn utf8_width(self) -> usize;
    fn class(self) -> Utf8Class;
}

impl Utf8Byte for u8 {
    #[inline(always)]
    fn utf8_width(self) -> usize {
        UTF8_CHAR_WIDTHS[self as usize] as usize
    }

    #[inline(always)]
    fn class(self) -> Utf8Class {
        if self < 128 {
            ASCII_CLASSES[self as usize]
        } else {
            Utf8Class::NON_ASCII
        }
    }
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
