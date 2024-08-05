use crate::Identifier;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Eof,
    Equal,
    Identifier(Identifier),
    Integer(i64),
    Plus,
    Star,
    LeftParenthesis,
    RightParenthesis,
    Float(f64),
}
