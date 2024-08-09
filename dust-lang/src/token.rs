//! Token and TokenOwned types.
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

/// Source code token.
#[derive(Debug, Serialize, Deserialize)]
pub enum Token<'src> {
    Eof,

    Identifier(&'src str),

    // Hard-coded values
    Boolean(bool),
    Float(f64),
    Integer(i64),
    String(&'src str),

    // Keywords
    IsEven,
    IsOdd,
    Length,
    ReadLine,
    WriteLine,

    // Symbols
    Comma,
    Dot,
    Equal,
    Greater,
    GreaterEqual,
    LeftParenthesis,
    LeftSquareBrace,
    Less,
    LessEqual,
    Minus,
    Plus,
    RightParenthesis,
    RightSquareBrace,
    Star,
}

impl<'src> Token<'src> {
    pub fn to_owned(&self) -> TokenOwned {
        match self {
            Token::Boolean(boolean) => TokenOwned::Boolean(*boolean),
            Token::Comma => TokenOwned::Comma,
            Token::Dot => TokenOwned::Dot,
            Token::Eof => TokenOwned::Eof,
            Token::Equal => TokenOwned::Equal,
            Token::Float(float) => TokenOwned::Float(*float),
            Token::Greater => TokenOwned::Greater,
            Token::GreaterEqual => TokenOwned::GreaterOrEqual,
            Token::Identifier(text) => TokenOwned::Identifier(text.to_string()),
            Token::Integer(integer) => TokenOwned::Integer(*integer),
            Token::IsEven => TokenOwned::IsEven,
            Token::IsOdd => TokenOwned::IsOdd,
            Token::LeftParenthesis => TokenOwned::LeftParenthesis,
            Token::LeftSquareBrace => TokenOwned::LeftSquareBrace,
            Token::Length => TokenOwned::Length,
            Token::Less => TokenOwned::Less,
            Token::LessEqual => TokenOwned::LessOrEqual,
            Token::Minus => TokenOwned::Minus,
            Token::Plus => TokenOwned::Plus,
            Token::ReadLine => TokenOwned::ReadLine,
            Token::RightParenthesis => TokenOwned::RightParenthesis,
            Token::RightSquareBrace => TokenOwned::RightSquareBrace,
            Token::Star => TokenOwned::Star,
            Token::String(text) => TokenOwned::String(text.to_string()),
            Token::WriteLine => TokenOwned::WriteLine,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Token::Boolean(_) => "boolean",
            Token::Comma => ",",
            Token::Dot => ".",
            Token::Eof => "EOF",
            Token::Equal => "=",
            Token::Float(_) => "float",
            Token::Greater => ">",
            Token::GreaterEqual => ">=",
            Token::Identifier(_) => "identifier",
            Token::Integer(_) => "integer",
            Token::IsEven => "is_even",
            Token::IsOdd => "is_odd",
            Token::LeftParenthesis => "(",
            Token::LeftSquareBrace => "[",
            Token::Length => "length",
            Token::Less => "<",
            Token::LessEqual => "<=",
            Token::Minus => "-",
            Token::Plus => "+",
            Token::ReadLine => "read_line",
            Token::RightParenthesis => ")",
            Token::RightSquareBrace => "]",
            Token::Star => "*",
            Token::String(_) => "string",
            Token::WriteLine => "write_line",
        }
    }
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'src> PartialEq for Token<'src> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Floats are compared by their bit representation.
            (Token::Float(left), Token::Float(right)) => left.to_bits() == right.to_bits(),

            // Compare all other variants normally.
            (Token::Boolean(left), Token::Boolean(right)) => left == right,
            (Token::Comma, Token::Comma) => true,
            (Token::Dot, Token::Dot) => true,
            (Token::Eof, Token::Eof) => true,
            (Token::Equal, Token::Equal) => true,
            (Token::Greater, Token::Greater) => true,
            (Token::GreaterEqual, Token::GreaterEqual) => true,
            (Token::Identifier(left), Token::Identifier(right)) => left == right,
            (Token::Integer(left), Token::Integer(right)) => left == right,
            (Token::IsEven, Token::IsEven) => true,
            (Token::IsOdd, Token::IsOdd) => true,
            (Token::LeftParenthesis, Token::LeftParenthesis) => true,
            (Token::LeftSquareBrace, Token::LeftSquareBrace) => true,
            (Token::Length, Token::Length) => true,
            (Token::Less, Token::Less) => true,
            (Token::LessEqual, Token::LessEqual) => true,
            (Token::Minus, Token::Minus) => true,
            (Token::Plus, Token::Plus) => true,
            (Token::ReadLine, Token::ReadLine) => true,
            (Token::RightParenthesis, Token::RightParenthesis) => true,
            (Token::RightSquareBrace, Token::RightSquareBrace) => true,
            (Token::Star, Token::Star) => true,
            (Token::String(left), Token::String(right)) => left == right,
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
    Boolean(bool),
    Float(f64),
    Integer(i64),
    String(String),

    // Keywords
    IsEven,
    IsOdd,
    Length,
    ReadLine,
    WriteLine,

    // Symbols
    Comma,
    Dot,
    Equal,
    Greater,
    GreaterOrEqual,
    LeftParenthesis,
    LeftSquareBrace,
    Less,
    LessOrEqual,
    Minus,
    Plus,
    RightParenthesis,
    RightSquareBrace,
    Star,
}

impl Display for TokenOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TokenOwned::Boolean(boolean) => write!(f, "{boolean}"),
            TokenOwned::Comma => Token::Comma.fmt(f),
            TokenOwned::Dot => Token::Dot.fmt(f),
            TokenOwned::Eof => Token::Eof.fmt(f),
            TokenOwned::Equal => Token::Equal.fmt(f),
            TokenOwned::Float(float) => write!(f, "{float}"),
            TokenOwned::Greater => Token::Greater.fmt(f),
            TokenOwned::GreaterOrEqual => Token::GreaterEqual.fmt(f),
            TokenOwned::Identifier(text) => write!(f, "{text}"),
            TokenOwned::Integer(integer) => write!(f, "{integer}"),
            TokenOwned::IsEven => Token::IsEven.fmt(f),
            TokenOwned::IsOdd => Token::IsOdd.fmt(f),
            TokenOwned::LeftParenthesis => Token::LeftParenthesis.fmt(f),
            TokenOwned::LeftSquareBrace => Token::LeftSquareBrace.fmt(f),
            TokenOwned::Length => Token::Length.fmt(f),
            TokenOwned::Less => Token::Less.fmt(f),
            TokenOwned::LessOrEqual => Token::LessEqual.fmt(f),
            TokenOwned::Minus => Token::Minus.fmt(f),
            TokenOwned::Plus => Token::Plus.fmt(f),
            TokenOwned::ReadLine => Token::ReadLine.fmt(f),
            TokenOwned::RightParenthesis => Token::RightParenthesis.fmt(f),
            TokenOwned::RightSquareBrace => Token::RightSquareBrace.fmt(f),
            TokenOwned::Star => Token::Star.fmt(f),
            TokenOwned::String(string) => write!(f, "{string}"),
            TokenOwned::WriteLine => Token::WriteLine.fmt(f),
        }
    }
}
