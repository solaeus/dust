use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Token {
    Eof,

    // Hard-coded values
    Boolean,
    Byte,
    Character,
    Float,
    Identifier,
    Integer,
    String,

    // Keywords
    Any,
    Async,
    Bool,
    Break,
    ByteKeyword,
    Cell,
    Const,
    Else,
    FloatKeyword,
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

    // Symbols
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
    LeftBrace,
    LeftBracket,
    LeftParenthesis,
    Less,
    LessEqual,
    Minus,
    MinusEqual,
    Percent,
    PercentEqual,
    Plus,
    PlusEqual,
    RightBrace,
    RightBracket,
    RightParenthesis,
    Semicolon,
    Slash,
    SlashEqual,
    StarEqual,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
