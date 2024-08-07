use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::Identifier;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Comma,
    Dot,
    Eof,
    Equal,
    Identifier(Identifier),
    ReservedIdentifier(ReservedIdentifier),
    Integer(i64),
    Plus,
    Star,
    LeftParenthesis,
    RightParenthesis,
    LeftSquareBrace,
    RightSquareBrace,
    Float(f64),
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReservedIdentifier {
    IsEven,
    IsOdd,
    Length,
}

impl Display for ReservedIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReservedIdentifier::IsEven => write!(f, "is_even"),
            ReservedIdentifier::IsOdd => write!(f, "is_odd"),
            ReservedIdentifier::Length => write!(f, "length"),
        }
    }
}
