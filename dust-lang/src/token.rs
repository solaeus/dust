use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

/// Source code token.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
        }
    }
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Token::Eof => write!(f, "EOF"),
            Token::Identifier(text) => write!(f, "{text}"),
            Token::Boolean(boolean) => write!(f, "{boolean}"),
            Token::Float(float) => write!(f, "{float}"),
            Token::Integer(integer) => write!(f, "{integer}"),
            Token::String(string) => write!(f, "{string}"),
            Token::IsEven => write!(f, "is_even"),
            Token::IsOdd => write!(f, "is_odd"),
            Token::Length => write!(f, "length"),
            Token::ReadLine => write!(f, "read_line"),
            Token::WriteLine => write!(f, "write_line"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::Equal => write!(f, "="),
            Token::Plus => write!(f, "+"),
            Token::Star => write!(f, "*"),
            Token::LeftParenthesis => write!(f, "("),
            Token::RightParenthesis => write!(f, ")"),
            Token::LeftSquareBrace => write!(f, "["),
            Token::RightSquareBrace => write!(f, "]"),
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
    Plus,
    RightParenthesis,
    RightSquareBrace,
    Star,
}
