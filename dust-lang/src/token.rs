use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::Identifier;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Token {
    Eof,

    Identifier(Identifier),

    // Hard-coded values
    Boolean(bool),
    Float(f64),
    Integer(i64),
    String(String),

    // Keywords
    IsEven,
    IsOdd,
    Length,

    // Symbols
    Comma,
    Dot,
    Equal,
    Plus,
    Star,
    LeftParenthesis,
    RightParenthesis,
    LeftSquareBrace,
    RightSquareBrace,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Token::Eof => write!(f, "EOF"),
            Token::Identifier(identifier) => write!(f, "{identifier}"),
            Token::Boolean(boolean) => write!(f, "{boolean}"),
            Token::Float(float) => write!(f, "{float}"),
            Token::Integer(integer) => write!(f, "{integer}"),
            Token::String(string) => write!(f, "{string}"),
            Token::IsEven => write!(f, "is_even"),
            Token::IsOdd => write!(f, "is_odd"),
            Token::Length => write!(f, "length"),
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
