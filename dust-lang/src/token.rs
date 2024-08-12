//! Token and TokenOwned types.
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

/// Source code token.
#[derive(Debug, Serialize, Deserialize)]
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
    Bool,
    Else,
    FloatKeyword,
    If,
    Int,
    IsEven,
    IsOdd,
    Length,
    ReadLine,
    Str,
    Struct,
    ToString,
    While,
    WriteLine,

    // Symbols
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
            Token::IsEven => TokenOwned::IsEven,
            Token::IsOdd => TokenOwned::IsOdd,
            Token::LeftCurlyBrace => TokenOwned::LeftCurlyBrace,
            Token::LeftParenthesis => TokenOwned::LeftParenthesis,
            Token::LeftSquareBrace => TokenOwned::LeftSquareBrace,
            Token::Length => TokenOwned::Length,
            Token::Less => TokenOwned::Less,
            Token::LessEqual => TokenOwned::LessOrEqual,
            Token::Minus => TokenOwned::Minus,
            Token::Percent => TokenOwned::Percent,
            Token::Plus => TokenOwned::Plus,
            Token::PlusEqual => TokenOwned::PlusEqual,
            Token::ReadLine => TokenOwned::ReadLine,
            Token::RightCurlyBrace => TokenOwned::RightCurlyBrace,
            Token::RightParenthesis => TokenOwned::RightParenthesis,
            Token::RightSquareBrace => TokenOwned::RightSquareBrace,
            Token::Semicolon => TokenOwned::Semicolon,
            Token::Star => TokenOwned::Star,
            Token::Slash => TokenOwned::Slash,
            Token::String(text) => TokenOwned::String(text.to_string()),
            Token::Str => TokenOwned::Str,
            Token::Struct => TokenOwned::Struct,
            Token::ToString => TokenOwned::ToString,
            Token::While => TokenOwned::While,
            Token::WriteLine => TokenOwned::WriteLine,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Token::Boolean(boolean_text) => boolean_text,
            Token::Float(float_text) => float_text,
            Token::Identifier(text) => text,
            Token::Integer(integer_text) => integer_text,
            Token::String(text) => text,

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
            Token::IsEven => "is_even",
            Token::IsOdd => "is_odd",
            Token::LeftCurlyBrace => "{",
            Token::LeftParenthesis => "(",
            Token::LeftSquareBrace => "[",
            Token::Length => "length",
            Token::Less => "<",
            Token::LessEqual => "<=",
            Token::Minus => "-",
            Token::Percent => "%",
            Token::Plus => "+",
            Token::PlusEqual => "+=",
            Token::ReadLine => "read_line",
            Token::RightCurlyBrace => "}",
            Token::RightParenthesis => ")",
            Token::RightSquareBrace => "]",
            Token::Semicolon => ";",
            Token::Star => "*",
            Token::Slash => "/",
            Token::Str => "str",
            Token::Struct => "struct",
            Token::ToString => "to_string",
            Token::While => "while",
            Token::WriteLine => "write_line",
        }
    }

    pub fn kind(&self) -> TokenKind {
        match self {
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
            Token::IsEven => TokenKind::IsEven,
            Token::IsOdd => TokenKind::IsOdd,
            Token::LeftCurlyBrace => TokenKind::LeftCurlyBrace,
            Token::LeftParenthesis => TokenKind::LeftParenthesis,
            Token::LeftSquareBrace => TokenKind::LeftSquareBrace,
            Token::Length => TokenKind::Length,
            Token::Less => TokenKind::Less,
            Token::LessEqual => TokenKind::LessOrEqual,
            Token::Minus => TokenKind::Minus,
            Token::Percent => TokenKind::Percent,
            Token::Plus => TokenKind::Plus,
            Token::PlusEqual => TokenKind::PlusEqual,
            Token::ReadLine => TokenKind::ReadLine,
            Token::RightCurlyBrace => TokenKind::RightCurlyBrace,
            Token::RightParenthesis => TokenKind::RightParenthesis,
            Token::RightSquareBrace => TokenKind::RightSquareBrace,
            Token::Semicolon => TokenKind::Semicolon,
            Token::Star => TokenKind::Star,
            Token::Slash => TokenKind::Slash,
            Token::Str => TokenKind::Str,
            Token::String(_) => TokenKind::String,
            Token::Struct => TokenKind::Struct,
            Token::ToString => TokenKind::ToString,
            Token::While => TokenKind::While,
            Token::WriteLine => TokenKind::WriteLine,
        }
    }

    pub fn is_eof(&self) -> bool {
        matches!(self, Token::Eof)
    }

    pub fn precedence(&self) -> u8 {
        match self {
            Token::Dot => 10,
            Token::LeftSquareBrace => 9,
            Token::Star | Token::Slash | Token::Percent => 8,
            Token::Minus => 7,
            Token::Plus => 6,
            Token::DoubleEqual
            | Token::Less
            | Token::LessEqual
            | Token::Greater
            | Token::GreaterEqual => 5,
            Token::DoubleAmpersand => 4,
            Token::DoublePipe => 3,
            Token::Equal | Token::PlusEqual => 2,
            Token::DoubleDot | Token::Semicolon => 1,
            _ => 0,
        }
    }

    pub fn is_left_associative(&self) -> bool {
        matches!(
            self,
            Token::DoubleAmpersand
                | Token::DoublePipe
                | Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Percent
        )
    }

    pub fn is_right_associative(&self) -> bool {
        matches!(self, Token::Equal | Token::PlusEqual)
    }

    pub fn is_prefix(&self) -> bool {
        matches!(self, Token::Bang | Token::Minus)
    }

    pub fn is_postfix(&self) -> bool {
        matches!(self, Token::LeftSquareBrace | Token::Semicolon)
    }
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<'src> PartialEq for Token<'src> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Token::Bang, Token::Bang) => true,
            (Token::Boolean(left), Token::Boolean(right)) => left == right,
            (Token::Comma, Token::Comma) => true,
            (Token::Dot, Token::Dot) => true,
            (Token::DoubleAmpersand, Token::DoubleAmpersand) => true,
            (Token::DoubleDot, Token::DoubleDot) => true,
            (Token::DoubleEqual, Token::DoubleEqual) => true,
            (Token::DoublePipe, Token::DoublePipe) => true,
            (Token::Else, Token::Else) => true,
            (Token::Eof, Token::Eof) => true,
            (Token::Equal, Token::Equal) => true,
            (Token::Float(left), Token::Float(right)) => left == right,
            (Token::Greater, Token::Greater) => true,
            (Token::GreaterEqual, Token::GreaterEqual) => true,
            (Token::Identifier(left), Token::Identifier(right)) => left == right,
            (Token::If, Token::If) => true,
            (Token::Integer(left), Token::Integer(right)) => left == right,
            (Token::IsEven, Token::IsEven) => true,
            (Token::IsOdd, Token::IsOdd) => true,
            (Token::LeftCurlyBrace, Token::LeftCurlyBrace) => true,
            (Token::LeftParenthesis, Token::LeftParenthesis) => true,
            (Token::LeftSquareBrace, Token::LeftSquareBrace) => true,
            (Token::Length, Token::Length) => true,
            (Token::Less, Token::Less) => true,
            (Token::LessEqual, Token::LessEqual) => true,
            (Token::Minus, Token::Minus) => true,
            (Token::Percent, Token::Percent) => true,
            (Token::Plus, Token::Plus) => true,
            (Token::PlusEqual, Token::PlusEqual) => true,
            (Token::ReadLine, Token::ReadLine) => true,
            (Token::RightCurlyBrace, Token::RightCurlyBrace) => true,
            (Token::RightParenthesis, Token::RightParenthesis) => true,
            (Token::RightSquareBrace, Token::RightSquareBrace) => true,
            (Token::Semicolon, Token::Semicolon) => true,
            (Token::Star, Token::Star) => true,
            (Token::Slash, Token::Slash) => true,
            (Token::String(left), Token::String(right)) => left == right,
            (Token::While, Token::While) => true,
            (Token::WriteLine, Token::WriteLine) => true,
            _ => false,
        }
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
    IsEven,
    IsOdd,
    Length,
    ReadLine,
    Str,
    ToString,
    While,
    WriteLine,

    // Symbols
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
            TokenOwned::Bang => Token::Bang.fmt(f),
            TokenOwned::Bool => write!(f, "bool"),
            TokenOwned::Boolean(boolean) => write!(f, "{boolean}"),
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
            TokenOwned::Float(float) => write!(f, "{float}"),
            TokenOwned::FloatKeyword => write!(f, "float"),
            TokenOwned::Greater => Token::Greater.fmt(f),
            TokenOwned::GreaterOrEqual => Token::GreaterEqual.fmt(f),
            TokenOwned::Identifier(text) => write!(f, "{text}"),
            TokenOwned::If => Token::If.fmt(f),
            TokenOwned::Int => write!(f, "int"),
            TokenOwned::Integer(integer) => write!(f, "{integer}"),
            TokenOwned::IsEven => Token::IsEven.fmt(f),
            TokenOwned::IsOdd => Token::IsOdd.fmt(f),
            TokenOwned::LeftCurlyBrace => Token::LeftCurlyBrace.fmt(f),
            TokenOwned::LeftParenthesis => Token::LeftParenthesis.fmt(f),
            TokenOwned::LeftSquareBrace => Token::LeftSquareBrace.fmt(f),
            TokenOwned::Length => Token::Length.fmt(f),
            TokenOwned::Less => Token::Less.fmt(f),
            TokenOwned::LessOrEqual => Token::LessEqual.fmt(f),
            TokenOwned::Minus => Token::Minus.fmt(f),
            TokenOwned::Percent => Token::Percent.fmt(f),
            TokenOwned::Plus => Token::Plus.fmt(f),
            TokenOwned::PlusEqual => Token::PlusEqual.fmt(f),
            TokenOwned::ReadLine => Token::ReadLine.fmt(f),
            TokenOwned::RightCurlyBrace => Token::RightCurlyBrace.fmt(f),
            TokenOwned::RightParenthesis => Token::RightParenthesis.fmt(f),
            TokenOwned::RightSquareBrace => Token::RightSquareBrace.fmt(f),
            TokenOwned::Semicolon => Token::Semicolon.fmt(f),
            TokenOwned::Star => Token::Star.fmt(f),
            TokenOwned::Slash => Token::Slash.fmt(f),
            TokenOwned::Str => write!(f, "str"),
            TokenOwned::String(string) => write!(f, "{string}"),
            TokenOwned::Struct => Token::Struct.fmt(f),
            TokenOwned::ToString => Token::ToString.fmt(f),
            TokenOwned::While => Token::While.fmt(f),
            TokenOwned::WriteLine => Token::WriteLine.fmt(f),
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
    Bool,
    Else,
    FloatKeyword,
    If,
    Int,
    IsEven,
    IsOdd,
    Length,
    ReadLine,
    Str,
    ToString,
    While,
    WriteLine,

    // Symbols
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
            TokenKind::Bang => Token::Bang.fmt(f),
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
            TokenKind::IsEven => Token::IsEven.fmt(f),
            TokenKind::IsOdd => Token::IsOdd.fmt(f),
            TokenKind::LeftCurlyBrace => Token::LeftCurlyBrace.fmt(f),
            TokenKind::LeftParenthesis => Token::LeftParenthesis.fmt(f),
            TokenKind::LeftSquareBrace => Token::LeftSquareBrace.fmt(f),
            TokenKind::Length => Token::Length.fmt(f),
            TokenKind::Less => Token::Less.fmt(f),
            TokenKind::LessOrEqual => Token::LessEqual.fmt(f),
            TokenKind::Minus => Token::Minus.fmt(f),
            TokenKind::Percent => Token::Percent.fmt(f),
            TokenKind::Plus => Token::Plus.fmt(f),
            TokenKind::PlusEqual => Token::PlusEqual.fmt(f),
            TokenKind::ReadLine => Token::ReadLine.fmt(f),
            TokenKind::RightCurlyBrace => Token::RightCurlyBrace.fmt(f),
            TokenKind::RightParenthesis => Token::RightParenthesis.fmt(f),
            TokenKind::RightSquareBrace => Token::RightSquareBrace.fmt(f),
            TokenKind::Semicolon => Token::Semicolon.fmt(f),
            TokenKind::Star => Token::Star.fmt(f),
            TokenKind::Str => write!(f, "str"),
            TokenKind::Slash => Token::Slash.fmt(f),
            TokenKind::String => write!(f, "string value"),
            TokenKind::Struct => Token::Struct.fmt(f),
            TokenKind::ToString => Token::ToString.fmt(f),
            TokenKind::While => Token::While.fmt(f),
            TokenKind::WriteLine => Token::WriteLine.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_tokens<'src>() -> [Token<'src>; 42] {
        [
            Token::Bang,
            Token::Boolean("true"),
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
            Token::Float("42.0"),
            Token::Greater,
            Token::GreaterEqual,
            Token::Identifier("foobar"),
            Token::If,
            Token::Integer("42"),
            Token::IsEven,
            Token::IsOdd,
            Token::LeftCurlyBrace,
            Token::LeftParenthesis,
            Token::LeftSquareBrace,
            Token::Length,
            Token::Less,
            Token::LessEqual,
            Token::Minus,
            Token::Percent,
            Token::Plus,
            Token::PlusEqual,
            Token::ReadLine,
            Token::RightCurlyBrace,
            Token::RightParenthesis,
            Token::RightSquareBrace,
            Token::Semicolon,
            Token::Star,
            Token::Slash,
            Token::String("foobar"),
            Token::Struct,
            Token::ToString,
            Token::While,
            Token::WriteLine,
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
