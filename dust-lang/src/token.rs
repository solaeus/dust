use crate::Identifier;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Comma,
    Eof,
    Equal,
    Identifier(Identifier),
    Integer(i64),
    Plus,
    Star,
    LeftParenthesis,
    RightParenthesis,
    LeftSquareBrace,
    RightSquareBrace,
    Float(f64),
}
