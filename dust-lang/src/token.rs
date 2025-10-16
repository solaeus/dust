use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::source::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.kind, self.span)
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TokenKind {
    // Characters that cannot be used in Dust source code
    #[default]
    Unknown,

    Eof,

    // Literals
    TrueValue,
    FalseValue,
    ByteValue,
    CharacterValue,
    FloatValue,
    IntegerValue,
    StringValue,

    // Names for variables, functions, types and modules
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
    Loop,
    Map,
    Mod,
    Mut,
    Pub,
    Return,
    Str,
    Struct,
    Use,
    While,

    // Operators and punctuation
    ArrowThin,
    Asterisk,
    AsteriskEqual,
    BangEqual,
    Bang,
    Colon,
    Comma,
    Dot,
    DoubleAmpersand,
    DoubleColon,
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

    // Comments
    LineComment,
    BlockComment,
    InnerLineDocComment,
    OuterLineDocComment,
    InnerBlockDocComment,
    OuterBlockDocComment,
}

impl TokenKind {
    pub fn is_comment(&self) -> bool {
        matches!(
            self,
            TokenKind::LineComment
                | TokenKind::BlockComment
                | TokenKind::InnerLineDocComment
                | TokenKind::OuterLineDocComment
                | TokenKind::InnerBlockDocComment
                | TokenKind::OuterBlockDocComment
        )
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TokenKind::Unknown => write!(f, "unknown token"),
            TokenKind::Eof => write!(f, "end of file"),
            TokenKind::TrueValue => write!(f, "true"),
            TokenKind::FalseValue => write!(f, "false"),
            TokenKind::ByteValue => write!(f, "byte value"),
            TokenKind::CharacterValue => write!(f, "character value"),
            TokenKind::FloatValue => write!(f, "float value"),
            TokenKind::IntegerValue => write!(f, "integer value"),
            TokenKind::StringValue => write!(f, "string value"),
            TokenKind::Identifier => write!(f, "identifier"),
            TokenKind::Any => write!(f, "'any' keyword"),
            TokenKind::Async => write!(f, "'async' keyword"),
            TokenKind::Bool => write!(f, "'bool' keyword"),
            TokenKind::Break => write!(f, "'break' keyword"),
            TokenKind::Byte => write!(f, "'byte' keyword"),
            TokenKind::Cell => write!(f, "'cell' keyword"),
            TokenKind::Char => write!(f, "'char' keyword"),
            TokenKind::Const => write!(f, "'const' keyword"),
            TokenKind::Else => write!(f, "'else' keyword"),
            TokenKind::Float => write!(f, "'float' keyword"),
            TokenKind::Fn => write!(f, "'fn' keyword"),
            TokenKind::If => write!(f, "'if' keyword"),
            TokenKind::Int => write!(f, "'int' keyword"),
            TokenKind::Let => write!(f, "'let' keyword"),
            TokenKind::Loop => write!(f, "'loop' keyword"),
            TokenKind::Map => write!(f, "'map' keyword"),
            TokenKind::Mod => write!(f, "'mod' keyword"),
            TokenKind::Mut => write!(f, "'mut' keyword"),
            TokenKind::Pub => write!(f, "'pub' keyword"),
            TokenKind::Return => write!(f, "'return' keyword"),
            TokenKind::Str => write!(f, "'str' keyword"),
            TokenKind::Struct => write!(f, "'struct' keyword"),
            TokenKind::Use => write!(f, "'use' keyword"),
            TokenKind::While => write!(f, "'while' keyword"),
            TokenKind::ArrowThin => write!(f, "-> symbol"),
            TokenKind::Asterisk => write!(f, "* symbol"),
            TokenKind::AsteriskEqual => write!(f, "*= symbol"),
            TokenKind::BangEqual => write!(f, "!= symbol"),
            TokenKind::Bang => write!(f, "! symbol"),
            TokenKind::Colon => write!(f, ": symbol"),
            TokenKind::Comma => write!(f, ", symbol"),
            TokenKind::Dot => write!(f, ". symbol"),
            TokenKind::DoubleAmpersand => write!(f, "&& symbol"),
            TokenKind::DoubleColon => write!(f, ":: symbol"),
            TokenKind::DoubleDot => write!(f, ".. symbol"),
            TokenKind::DoubleEqual => write!(f, "== symbol"),
            TokenKind::DoublePipe => write!(f, "|| symbol"),
            TokenKind::Equal => write!(f, "= symbol"),
            TokenKind::Greater => write!(f, "> symbol"),
            TokenKind::GreaterEqual => write!(f, ">= symbol"),
            TokenKind::LeftCurlyBrace => write!(f, "{{ symbol"),
            TokenKind::LeftSquareBracket => write!(f, "[ symbol"),
            TokenKind::LeftParenthesis => write!(f, "( symbol"),
            TokenKind::Less => write!(f, "< symbol"),
            TokenKind::LessEqual => write!(f, "<= symbol"),
            TokenKind::Minus => write!(f, "- symbol"),
            TokenKind::MinusEqual => write!(f, "-= symbol"),
            TokenKind::Percent => write!(f, "% symbol"),
            TokenKind::PercentEqual => write!(f, "%= symbol"),
            TokenKind::Plus => write!(f, "+ symbol"),
            TokenKind::PlusEqual => write!(f, "+= symbol"),
            TokenKind::RightCurlyBrace => write!(f, "}} symbol"),
            TokenKind::RightSquareBracket => write!(f, "] symbol"),
            TokenKind::RightParenthesis => write!(f, ") symbol"),
            TokenKind::Semicolon => write!(f, "; symbol"),
            TokenKind::Slash => write!(f, "/ symbol"),
            TokenKind::SlashEqual => write!(f, "/= symbol"),
            TokenKind::LineComment => write!(f, "line comment"),
            TokenKind::BlockComment => write!(f, "block comment"),
            TokenKind::InnerLineDocComment => write!(f, "inner line doc comment"),
            TokenKind::OuterLineDocComment => write!(f, "outer line doc comment"),
            TokenKind::InnerBlockDocComment => write!(f, "inner block doc comment"),
            TokenKind::OuterBlockDocComment => write!(f, "outer block doc comment"),
        }
    }
}
