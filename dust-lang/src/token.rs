use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Token {
    // End of file
    Eof,

    // Represents characrers that cannot be used in Dust source code
    Unknown,

    // Hard-coded values
    TrueValue,
    FalseValue,
    ByteValue,
    CharacterValue,
    FloatValue,
    IntegerValue,
    StringValue,

    // Paths to declared items or variables
    Identifier,

    // Keywords
    Any,
    Async,
    Bool,
    Break,
    Byte,
    Cell,
    Char,
    Const,
    Else,
    Float,
    Fn,
    If,
    Int,
    Let,
    List,
    Loop,
    Map,
    Mod,
    Mut,
    Return,
    Str,
    Struct,
    Use,
    While,

    // Symbols (operators and punctuation)
    ArrowThin,
    Asterisk,
    BangEqual,
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
    LeftSquareBracket,
    LeftParenthesis,
    Less,
    LessEqual,
    Minus,
    MinusEqual,
    Percent,
    PercentEqual,
    Plus,
    PlusEqual,
    RightCurlyBrace,
    RightSquareBracket,
    RightParenthesis,
    Semicolon,
    Slash,
    SlashEqual,
    StarEqual,
}

impl Token {
    pub fn is_type(&self) -> bool {
        matches!(
            self,
            Token::TrueValue
                | Token::FalseValue
                | Token::ByteValue
                | Token::CharacterValue
                | Token::FloatValue
                | Token::IntegerValue
                | Token::StringValue
        )
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
