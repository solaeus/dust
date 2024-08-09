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
    LeftParenthesis,
    LeftSquareBrace,
    Minus,
    Plus,
    RightParenthesis,
    RightSquareBrace,
    Star,
}

impl<'src> Token<'src> {
    pub fn to_owned(&self) -> TokenOwned {
        match self {
            Token::Eof => TokenOwned::Eof,
            Token::Identifier(text) => TokenOwned::Identifier(text.to_string()),
            Token::Boolean(boolean) => TokenOwned::Boolean(*boolean),
            Token::Float(float) => TokenOwned::Float(*float),
            Token::Integer(integer) => TokenOwned::Integer(*integer),
            Token::String(text) => TokenOwned::String(text.to_string()),
            Token::IsEven => TokenOwned::IsEven,
            Token::IsOdd => TokenOwned::IsOdd,
            Token::Length => TokenOwned::Length,
            Token::ReadLine => TokenOwned::ReadLine,
            Token::WriteLine => TokenOwned::WriteLine,
            Token::Comma => TokenOwned::Comma,
            Token::Dot => TokenOwned::Dot,
            Token::Equal => TokenOwned::Equal,
            Token::Plus => TokenOwned::Plus,
            Token::Star => TokenOwned::Star,
            Token::LeftParenthesis => TokenOwned::LeftParenthesis,
            Token::RightParenthesis => TokenOwned::RightParenthesis,
            Token::LeftSquareBrace => TokenOwned::LeftSquareBrace,
            Token::RightSquareBrace => TokenOwned::RightSquareBrace,
            Token::Minus => TokenOwned::Minus,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Token::Eof => "EOF",
            Token::Identifier(_) => "identifier",
            Token::Boolean(_) => "boolean",
            Token::Float(_) => "float",
            Token::Integer(_) => "integer",
            Token::String(_) => "string",
            Token::IsEven => "is_even",
            Token::IsOdd => "is_odd",
            Token::Length => "length",
            Token::ReadLine => "read_line",
            Token::WriteLine => "write_line",
            Token::Comma => ",",
            Token::Dot => ".",
            Token::Equal => "=",
            Token::Plus => "+",
            Token::Star => "*",
            Token::LeftParenthesis => "(",
            Token::RightParenthesis => ")",
            Token::LeftSquareBrace => "[",
            Token::RightSquareBrace => "]",
            Token::Minus => "-",
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
            (Token::Eof, Token::Eof) => true,
            (Token::Identifier(left), Token::Identifier(right)) => left == right,
            (Token::Boolean(left), Token::Boolean(right)) => left == right,
            (Token::Integer(left), Token::Integer(right)) => left == right,
            (Token::String(left), Token::String(right)) => left == right,
            (Token::IsEven, Token::IsEven) => true,
            (Token::IsOdd, Token::IsOdd) => true,
            (Token::Length, Token::Length) => true,
            (Token::ReadLine, Token::ReadLine) => true,
            (Token::WriteLine, Token::WriteLine) => true,
            (Token::Comma, Token::Comma) => true,
            (Token::Dot, Token::Dot) => true,
            (Token::Equal, Token::Equal) => true,
            (Token::Plus, Token::Plus) => true,
            (Token::Star, Token::Star) => true,
            (Token::LeftParenthesis, Token::LeftParenthesis) => true,
            (Token::RightParenthesis, Token::RightParenthesis) => true,
            (Token::LeftSquareBrace, Token::LeftSquareBrace) => true,
            (Token::RightSquareBrace, Token::RightSquareBrace) => true,
            (Token::Minus, Token::Minus) => true,
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
    LeftParenthesis,
    LeftSquareBrace,
    Minus,
    Plus,
    RightParenthesis,
    RightSquareBrace,
    Star,
}

impl Display for TokenOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TokenOwned::Eof => Token::Eof.fmt(f),
            TokenOwned::Identifier(text) => write!(f, "{text}"),
            TokenOwned::Boolean(boolean) => write!(f, "{boolean}"),
            TokenOwned::Float(float) => write!(f, "{float}"),
            TokenOwned::Integer(integer) => write!(f, "{integer}"),
            TokenOwned::String(string) => write!(f, "{string}"),
            TokenOwned::IsEven => Token::IsEven.fmt(f),
            TokenOwned::IsOdd => Token::IsOdd.fmt(f),
            TokenOwned::Length => Token::Length.fmt(f),
            TokenOwned::ReadLine => Token::ReadLine.fmt(f),
            TokenOwned::WriteLine => Token::WriteLine.fmt(f),
            TokenOwned::Comma => Token::Comma.fmt(f),
            TokenOwned::Dot => Token::Dot.fmt(f),
            TokenOwned::Equal => Token::Equal.fmt(f),
            TokenOwned::Plus => Token::Plus.fmt(f),
            TokenOwned::Star => Token::Star.fmt(f),
            TokenOwned::LeftParenthesis => Token::LeftParenthesis.fmt(f),
            TokenOwned::RightParenthesis => Token::RightParenthesis.fmt(f),
            TokenOwned::LeftSquareBrace => Token::LeftSquareBrace.fmt(f),
            TokenOwned::RightSquareBrace => Token::RightSquareBrace.fmt(f),
            TokenOwned::Minus => Token::Minus.fmt(f),
        }
    }
}
