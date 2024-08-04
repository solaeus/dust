use crate::Identifier;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Eof,
    Equal,
    Identifier(Identifier),
    Number(i64),
    Plus,
    Star,
    LeftParenthesis,
    RightParenthesis,
}
