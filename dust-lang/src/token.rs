//! Token and TokenOwned types.
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

pub struct Raw<'src> {
    data: &'src str,
}

/// Source code token.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Token<'src> {
    // End of file
    Eof,

    // Hard-coded values
    Boolean(&'src str),
    Float(&'src str),
    Identifier(&'src str),
    Integer(&'src str),
    String(&'src str),

    // Keywords
    Async,
    Bool,
    Else,
    FloatKeyword,
    If,
    Int,
    Let,
    Map,
    Mut,
    Str,
    Struct,
    While,

    // Symbols
    BangEqual,
    Bang,
    Colon,
    Comma,
    Dot,
    DoubleAmpersand,
    DoubleDot,
    DoubleEqual,
    DoublePipe,
    Equal,
    Greater,
    GreaterEqual,
    LeftCurlyBrace,
    LeftParenthesis,
    LeftSquareBrace,
    Less,
    LessEqual,
    Minus,
    MinusEqual,
    Percent,
    Plus,
    PlusEqual,
    RightCurlyBrace,
    RightParenthesis,
    RightSquareBrace,
    Semicolon,
    Slash,
    Star,
}

impl<'src> Token<'src> {
    pub fn to_owned(&self) -> TokenOwned {
        match self {
            Token::Async => TokenOwned::Async,
            Token::BangEqual => TokenOwned::BangEqual,
            Token::Bang => TokenOwned::Bang,
            Token::Bool => TokenOwned::Bool,
            Token::Boolean(boolean) => TokenOwned::Boolean(boolean.to_string()),
            Token::Colon => TokenOwned::Colon,
            Token::Comma => TokenOwned::Comma,
            Token::Dot => TokenOwned::Dot,
            Token::DoubleAmpersand => TokenOwned::DoubleAmpersand,
            Token::DoubleDot => TokenOwned::DoubleDot,
            Token::DoubleEqual => TokenOwned::DoubleEqual,
            Token::DoublePipe => TokenOwned::DoublePipe,
            Token::Else => TokenOwned::Else,
            Token::Eof => TokenOwned::Eof,
            Token::Equal => TokenOwned::Equal,
            Token::Float(float) => TokenOwned::Float(float.to_string()),
            Token::FloatKeyword => TokenOwned::FloatKeyword,
            Token::Greater => TokenOwned::Greater,
            Token::GreaterEqual => TokenOwned::GreaterOrEqual,
            Token::Identifier(text) => TokenOwned::Identifier(text.to_string()),
            Token::If => TokenOwned::If,
            Token::Int => TokenOwned::Int,
            Token::Integer(integer) => TokenOwned::Integer(integer.to_string()),
            Token::LeftCurlyBrace => TokenOwned::LeftCurlyBrace,
            Token::LeftParenthesis => TokenOwned::LeftParenthesis,
            Token::LeftSquareBrace => TokenOwned::LeftSquareBrace,
            Token::Let => TokenOwned::Let,
            Token::Less => TokenOwned::Less,
            Token::LessEqual => TokenOwned::LessOrEqual,
            Token::Map => TokenOwned::Map,
            Token::Minus => TokenOwned::Minus,
            Token::MinusEqual => TokenOwned::MinusEqual,
            Token::Mut => TokenOwned::Mut,
            Token::Percent => TokenOwned::Percent,
            Token::Plus => TokenOwned::Plus,
            Token::PlusEqual => TokenOwned::PlusEqual,
            Token::RightCurlyBrace => TokenOwned::RightCurlyBrace,
            Token::RightParenthesis => TokenOwned::RightParenthesis,
            Token::RightSquareBrace => TokenOwned::RightSquareBrace,
            Token::Semicolon => TokenOwned::Semicolon,
            Token::Star => TokenOwned::Star,
            Token::Slash => TokenOwned::Slash,
            Token::String(text) => TokenOwned::String(text.to_string()),
            Token::Str => TokenOwned::Str,
            Token::Struct => TokenOwned::Struct,
            Token::While => TokenOwned::While,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Token::Boolean(boolean_text) => boolean_text,
            Token::Float(float_text) => float_text,
            Token::Identifier(text) => text,
            Token::Integer(integer_text) => integer_text,
            Token::String(text) => text,

            Token::Async => "async",
            Token::BangEqual => "!=",
            Token::Bang => "!",
            Token::Bool => "bool",
            Token::Colon => ":",
            Token::Comma => ",",
            Token::Dot => ".",
            Token::DoubleAmpersand => "&&",
            Token::DoubleDot => "..",
            Token::DoubleEqual => "==",
            Token::DoublePipe => "||",
            Token::Else => "else",
            Token::Eof => "EOF",
            Token::Equal => "=",
            Token::FloatKeyword => "float",
            Token::Greater => ">",
            Token::GreaterEqual => ">=",
            Token::If => "if",
            Token::Int => "int",
            Token::LeftCurlyBrace => "{",
            Token::LeftParenthesis => "(",
            Token::LeftSquareBrace => "[",
            Token::Let => "let",
            Token::Less => "<",
            Token::LessEqual => "<=",
            Token::Map => "map",
            Token::Minus => "-",
            Token::MinusEqual => "-=",
            Token::Mut => "mut",
            Token::Percent => "%",
            Token::Plus => "+",
            Token::PlusEqual => "+=",
            Token::RightCurlyBrace => "}",
            Token::RightParenthesis => ")",
            Token::RightSquareBrace => "]",
            Token::Semicolon => ";",
            Token::Star => "*",
            Token::Slash => "/",
            Token::Str => "str",
            Token::Struct => "struct",
            Token::While => "while",
        }
    }

    pub fn kind(&self) -> TokenKind {
        match self {
            Token::Async => TokenKind::Async,
            Token::BangEqual => TokenKind::BangEqual,
            Token::Bang => TokenKind::Bang,
            Token::Bool => TokenKind::Bool,
            Token::Boolean(_) => TokenKind::Boolean,
            Token::Colon => TokenKind::Colon,
            Token::Comma => TokenKind::Comma,
            Token::Dot => TokenKind::Dot,
            Token::DoubleAmpersand => TokenKind::DoubleAmpersand,
            Token::DoubleDot => TokenKind::DoubleDot,
            Token::DoubleEqual => TokenKind::DoubleEqual,
            Token::DoublePipe => TokenKind::DoublePipe,
            Token::Else => TokenKind::Else,
            Token::Eof => TokenKind::Eof,
            Token::Equal => TokenKind::Equal,
            Token::Float(_) => TokenKind::Float,
            Token::FloatKeyword => TokenKind::FloatKeyword,
            Token::Greater => TokenKind::Greater,
            Token::GreaterEqual => TokenKind::GreaterOrEqual,
            Token::Identifier(_) => TokenKind::Identifier,
            Token::If => TokenKind::If,
            Token::Int => TokenKind::Int,
            Token::Integer(_) => TokenKind::Integer,
            Token::LeftCurlyBrace => TokenKind::LeftCurlyBrace,
            Token::LeftParenthesis => TokenKind::LeftParenthesis,
            Token::LeftSquareBrace => TokenKind::LeftSquareBrace,
            Token::Let => TokenKind::Let,
            Token::Less => TokenKind::Less,
            Token::LessEqual => TokenKind::LessOrEqual,
            Token::Map => TokenKind::Map,
            Token::Minus => TokenKind::Minus,
            Token::MinusEqual => TokenKind::MinusEqual,
            Token::Mut => TokenKind::Mut,
            Token::Percent => TokenKind::Percent,
            Token::Plus => TokenKind::Plus,
            Token::PlusEqual => TokenKind::PlusEqual,
            Token::RightCurlyBrace => TokenKind::RightCurlyBrace,
            Token::RightParenthesis => TokenKind::RightParenthesis,
            Token::RightSquareBrace => TokenKind::RightSquareBrace,
            Token::Semicolon => TokenKind::Semicolon,
            Token::Star => TokenKind::Star,
            Token::Slash => TokenKind::Slash,
            Token::Str => TokenKind::Str,
            Token::String(_) => TokenKind::String,
            Token::Struct => TokenKind::Struct,
            Token::While => TokenKind::While,
        }
    }

    pub fn is_eof(&self) -> bool {
        matches!(self, Token::Eof)
    }

    pub fn precedence(&self) -> u8 {
        match self {
            Token::Dot => 9,
            Token::LeftParenthesis | Token::LeftSquareBrace => 8,
            Token::Star | Token::Slash | Token::Percent => 7,
            Token::Minus | Token::Plus => 6,
            Token::DoubleEqual
            | Token::Less
            | Token::LessEqual
            | Token::Greater
            | Token::GreaterEqual => 5,
            Token::DoubleAmpersand => 4,
            Token::DoublePipe => 3,
            Token::DoubleDot => 2,
            Token::Equal | Token::MinusEqual | Token::PlusEqual => 1,
            _ => 0,
        }
    }

    pub fn is_left_associative(&self) -> bool {
        matches!(
            self,
            Token::Dot
                | Token::DoubleAmpersand
                | Token::DoublePipe
                | Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Percent
        )
    }

    pub fn is_right_associative(&self) -> bool {
        matches!(self, Token::Equal | Token::MinusEqual | Token::PlusEqual)
    }

    pub fn is_prefix(&self) -> bool {
        matches!(self, Token::Bang | Token::Minus)
    }

    pub fn is_postfix(&self) -> bool {
        matches!(
            self,
            Token::Dot | Token::LeftCurlyBrace | Token::LeftParenthesis | Token::LeftSquareBrace
        )
    }
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Owned version of `Token`, which owns all the strings.
///
/// This is used for errors.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum TokenOwned {
    Eof,

    Identifier(String),

    // Hard-coded values
    Boolean(String),
    Float(String),
    Integer(String),
    String(String),

    // Keywords
    Bool,
    Else,
    FloatKeyword,
    If,
    Int,
    Let,
    Map,
    Mut,
    Str,
    While,

    // Symbols
    Async,
    Bang,
    BangEqual,
    Colon,
    Comma,
    Dot,
    DoubleAmpersand,
    DoubleDot,
    DoubleEqual,
    DoublePipe,
    Equal,
    Greater,
    GreaterOrEqual,
    LeftCurlyBrace,
    LeftParenthesis,
    LeftSquareBrace,
    Less,
    LessOrEqual,
    Minus,
    MinusEqual,
    Percent,
    Plus,
    PlusEqual,
    RightCurlyBrace,
    RightParenthesis,
    RightSquareBrace,
    Semicolon,
    Star,
    Struct,
    Slash,
}

impl Display for TokenOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TokenOwned::Async => Token::Async.fmt(f),
            TokenOwned::Bang => Token::Bang.fmt(f),
            TokenOwned::BangEqual => Token::BangEqual.fmt(f),
            TokenOwned::Bool => Token::Bool.fmt(f),
            TokenOwned::Boolean(boolean) => Token::Boolean(boolean).fmt(f),
            TokenOwned::Colon => Token::Colon.fmt(f),
            TokenOwned::Comma => Token::Comma.fmt(f),
            TokenOwned::Dot => Token::Dot.fmt(f),
            TokenOwned::DoubleAmpersand => Token::DoubleAmpersand.fmt(f),
            TokenOwned::DoubleDot => Token::DoubleDot.fmt(f),
            TokenOwned::DoubleEqual => Token::DoubleEqual.fmt(f),
            TokenOwned::DoublePipe => Token::DoublePipe.fmt(f),
            TokenOwned::Else => Token::Else.fmt(f),
            TokenOwned::Eof => Token::Eof.fmt(f),
            TokenOwned::Equal => Token::Equal.fmt(f),
            TokenOwned::Float(float) => Token::Float(float).fmt(f),
            TokenOwned::FloatKeyword => Token::FloatKeyword.fmt(f),
            TokenOwned::Greater => Token::Greater.fmt(f),
            TokenOwned::GreaterOrEqual => Token::GreaterEqual.fmt(f),
            TokenOwned::Identifier(text) => Token::Identifier(text).fmt(f),
            TokenOwned::If => Token::If.fmt(f),
            TokenOwned::Int => Token::Int.fmt(f),
            TokenOwned::Integer(integer) => Token::Integer(integer).fmt(f),
            TokenOwned::LeftCurlyBrace => Token::LeftCurlyBrace.fmt(f),
            TokenOwned::LeftParenthesis => Token::LeftParenthesis.fmt(f),
            TokenOwned::LeftSquareBrace => Token::LeftSquareBrace.fmt(f),
            TokenOwned::Let => Token::Let.fmt(f),
            TokenOwned::Less => Token::Less.fmt(f),
            TokenOwned::LessOrEqual => Token::LessEqual.fmt(f),
            TokenOwned::Map => Token::Map.fmt(f),
            TokenOwned::Minus => Token::Minus.fmt(f),
            TokenOwned::MinusEqual => Token::MinusEqual.fmt(f),
            TokenOwned::Mut => Token::Mut.fmt(f),
            TokenOwned::Percent => Token::Percent.fmt(f),
            TokenOwned::Plus => Token::Plus.fmt(f),
            TokenOwned::PlusEqual => Token::PlusEqual.fmt(f),
            TokenOwned::RightCurlyBrace => Token::RightCurlyBrace.fmt(f),
            TokenOwned::RightParenthesis => Token::RightParenthesis.fmt(f),
            TokenOwned::RightSquareBrace => Token::RightSquareBrace.fmt(f),
            TokenOwned::Semicolon => Token::Semicolon.fmt(f),
            TokenOwned::Star => Token::Star.fmt(f),
            TokenOwned::Slash => Token::Slash.fmt(f),
            TokenOwned::Str => Token::Str.fmt(f),
            TokenOwned::String(string) => write!(f, "{string}"),
            TokenOwned::Struct => Token::Struct.fmt(f),
            TokenOwned::While => Token::While.fmt(f),
        }
    }
}

/// Token representation that holds no data.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum TokenKind {
    Eof,

    Identifier,

    // Hard-coded values
    Boolean,
    Float,
    Integer,
    String,

    // Keywords
    Async,
    Bool,
    Else,
    FloatKeyword,
    If,
    Int,
    Let,
    Map,
    Str,
    While,

    // Symbols
    BangEqual,
    Bang,
    Colon,
    Comma,
    Dot,
    DoubleAmpersand,
    DoubleDot,
    DoubleEqual,
    DoublePipe,
    Equal,
    Greater,
    GreaterOrEqual,
    LeftCurlyBrace,
    LeftParenthesis,
    LeftSquareBrace,
    Less,
    LessOrEqual,
    Minus,
    MinusEqual,
    Mut,
    Percent,
    Plus,
    PlusEqual,
    RightCurlyBrace,
    RightParenthesis,
    RightSquareBrace,
    Semicolon,
    Star,
    Struct,
    Slash,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Async => Token::Async.fmt(f),
            TokenKind::Bang => Token::Bang.fmt(f),
            TokenKind::BangEqual => Token::BangEqual.fmt(f),
            TokenKind::Bool => Token::Bool.fmt(f),
            TokenKind::Boolean => write!(f, "boolean value"),
            TokenKind::Colon => Token::Colon.fmt(f),
            TokenKind::Comma => Token::Comma.fmt(f),
            TokenKind::Dot => Token::Dot.fmt(f),
            TokenKind::DoubleAmpersand => Token::DoubleAmpersand.fmt(f),
            TokenKind::DoubleDot => Token::DoubleDot.fmt(f),
            TokenKind::DoubleEqual => Token::DoubleEqual.fmt(f),
            TokenKind::DoublePipe => Token::DoublePipe.fmt(f),
            TokenKind::Else => Token::Else.fmt(f),
            TokenKind::Eof => Token::Eof.fmt(f),
            TokenKind::Equal => Token::Equal.fmt(f),
            TokenKind::Float => write!(f, "float value"),
            TokenKind::FloatKeyword => Token::FloatKeyword.fmt(f),
            TokenKind::Greater => Token::Greater.fmt(f),
            TokenKind::GreaterOrEqual => Token::GreaterEqual.fmt(f),
            TokenKind::Identifier => write!(f, "identifier"),
            TokenKind::If => Token::If.fmt(f),
            TokenKind::Int => Token::Int.fmt(f),
            TokenKind::Integer => write!(f, "integer value"),
            TokenKind::LeftCurlyBrace => Token::LeftCurlyBrace.fmt(f),
            TokenKind::LeftParenthesis => Token::LeftParenthesis.fmt(f),
            TokenKind::LeftSquareBrace => Token::LeftSquareBrace.fmt(f),
            TokenKind::Let => Token::Let.fmt(f),
            TokenKind::Less => Token::Less.fmt(f),
            TokenKind::LessOrEqual => Token::LessEqual.fmt(f),
            TokenKind::Map => Token::Map.fmt(f),
            TokenKind::Minus => Token::Minus.fmt(f),
            TokenKind::MinusEqual => Token::MinusEqual.fmt(f),
            TokenKind::Mut => Token::Mut.fmt(f),
            TokenKind::Percent => Token::Percent.fmt(f),
            TokenKind::Plus => Token::Plus.fmt(f),
            TokenKind::PlusEqual => Token::PlusEqual.fmt(f),
            TokenKind::RightCurlyBrace => Token::RightCurlyBrace.fmt(f),
            TokenKind::RightParenthesis => Token::RightParenthesis.fmt(f),
            TokenKind::RightSquareBrace => Token::RightSquareBrace.fmt(f),
            TokenKind::Semicolon => Token::Semicolon.fmt(f),
            TokenKind::Star => Token::Star.fmt(f),
            TokenKind::Str => write!(f, "str"),
            TokenKind::Slash => Token::Slash.fmt(f),
            TokenKind::String => write!(f, "string value"),
            TokenKind::Struct => Token::Struct.fmt(f),
            TokenKind::While => Token::While.fmt(f),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn all_tokens<'src>() -> [Token<'src>; 46] {
        [
            Token::Identifier("foobar"),
            Token::Boolean("true"),
            Token::Float("1.0"),
            Token::Integer("1"),
            Token::String("string"),
            Token::Async,
            Token::Bang,
            Token::BangEqual,
            Token::Bool,
            Token::Colon,
            Token::Comma,
            Token::Dot,
            Token::DoubleAmpersand,
            Token::DoubleDot,
            Token::DoubleEqual,
            Token::DoublePipe,
            Token::Else,
            Token::Eof,
            Token::Equal,
            Token::FloatKeyword,
            Token::Greater,
            Token::GreaterEqual,
            Token::If,
            Token::Int,
            Token::LeftCurlyBrace,
            Token::LeftParenthesis,
            Token::LeftSquareBrace,
            Token::Less,
            Token::LessEqual,
            Token::Let,
            Token::Map,
            Token::Minus,
            Token::MinusEqual,
            Token::Mut,
            Token::Percent,
            Token::Plus,
            Token::PlusEqual,
            Token::RightCurlyBrace,
            Token::RightParenthesis,
            Token::RightSquareBrace,
            Token::Semicolon,
            Token::Slash,
            Token::Star,
            Token::Str,
            Token::Struct,
            Token::While,
        ]
    }

    #[test]
    fn token_displays() {
        for token in all_tokens().iter() {
            let display = token.to_string();

            assert_eq!(display, token.to_owned().to_string());

            if let Token::Boolean(_)
            | Token::Float(_)
            | Token::Identifier(_)
            | Token::Integer(_)
            | Token::String(_) = token
            {
                continue;
            } else {
                assert_eq!(display, token.kind().to_string());
            }
        }
    }
}
