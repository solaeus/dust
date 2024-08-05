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

#[derive(Debug, PartialEq, Clone)]
pub enum ReservedIdentifier {
    IsEven,
    IsOdd,
    Length,
}
