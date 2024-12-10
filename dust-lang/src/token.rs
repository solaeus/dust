//! Token, TokenOwned and TokenKind types.
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

macro_rules! define_tokens {
    ($($variant:ident $(($data_type:ty))?),+ $(,)?) => {
        /// Source token.
        ///
        /// This is a borrowed type, i.e. some variants contain references to the source text.
        #[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Default, Serialize, Deserialize)]
        pub enum Token<'src> {
            #[default]
            Eof,
            $(
                $variant $(($data_type))?,
            )*
        }

        #[derive(Debug, PartialEq, Clone)]
        /// Data-less representation of a source token.
        ///
        /// If a [Token] borrows from the source text, its TokenKind omits the data.
        pub enum TokenKind {
            Eof,
            $(
                $variant,
            )*
        }
    };
}

define_tokens! {
    // Hard-coded values
    Boolean(&'src str),
    Byte(&'src str),
    Character(char),
    Float(&'src str),
    Identifier(&'src str),
    Integer(&'src str),
    String(&'src str),

    // Keywords
    Async,
    Bool,
    Break,
    Else,
    FloatKeyword,
    Fn,
    If,
    Int,
    Let,
    Loop,
    Map,
    Mut,
    Return,
    Str,
    Struct,
    While,

    // Symbols
    ArrowThin,
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
    Star,
    StarEqual,
}

impl<'src> Token<'src> {
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match self {
            Token::Eof => 0,
            Token::Boolean(text) => text.len(),
            Token::Byte(_) => 3,
            Token::Character(_) => 3,
            Token::Float(text) => text.len(),
            Token::Identifier(text) => text.len(),
            Token::Integer(text) => text.len(),
            Token::String(text) => text.len() + 2,
            Token::Async => 5,
            Token::ArrowThin => 2,
            Token::Bool => 4,
            Token::Break => 5,
            Token::Else => 4,
            Token::FloatKeyword => 5,
            Token::Fn => 2,
            Token::If => 2,
            Token::Int => 3,
            Token::Let => 3,
            Token::Loop => 4,
            Token::Map => 3,
            Token::Mut => 3,
            Token::Str => 3,
            Token::Struct => 6,
            Token::While => 5,
            Token::BangEqual => 2,
            Token::Bang => 1,
            Token::Colon => 1,
            Token::Comma => 1,
            Token::Dot => 1,
            Token::DoubleAmpersand => 2,
            Token::DoubleDot => 2,
            Token::DoubleEqual => 2,
            Token::DoublePipe => 2,
            Token::Equal => 1,
            Token::Greater => 1,
            Token::GreaterEqual => 2,
            Token::LeftBrace => 1,
            Token::LeftParenthesis => 1,
            Token::LeftBracket => 1,
            Token::Less => 1,
            Token::LessEqual => 2,
            Token::Minus => 1,
            Token::MinusEqual => 2,
            Token::Percent => 1,
            Token::PercentEqual => 2,
            Token::Plus => 1,
            Token::PlusEqual => 2,
            Token::Return => 6,
            Token::RightBrace => 1,
            Token::RightParenthesis => 1,
            Token::RightBracket => 1,
            Token::Semicolon => 1,
            Token::Slash => 1,
            Token::SlashEqual => 2,
            Token::Star => 1,
            Token::StarEqual => 2,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Token::Eof => "",
            Token::Boolean(text) => text,
            Token::Byte(text) => text,
            Token::Character(_) => "character token",
            Token::Float(text) => text,
            Token::Identifier(text) => text,
            Token::Integer(text) => text,
            Token::String(text) => text,
            Token::Async => "async",
            Token::ArrowThin => "->",
            Token::Bool => "bool",
            Token::Break => "break",
            Token::Else => "else",
            Token::FloatKeyword => "float",
            Token::Fn => "fn",
            Token::If => "if",
            Token::Int => "int",
            Token::Let => "let",
            Token::Loop => "loop",
            Token::Map => "map",
            Token::Mut => "mut",
            Token::Str => "str",
            Token::Struct => "struct",
            Token::While => "while",
            Token::BangEqual => "!=",
            Token::Bang => "!",
            Token::Colon => ":",
            Token::Comma => ",",
            Token::Dot => ".",
            Token::DoubleAmpersand => "&&",
            Token::DoubleDot => "..",
            Token::DoubleEqual => "==",
            Token::DoublePipe => "||",
            Token::Equal => "=",
            Token::Greater => ">",
            Token::GreaterEqual => ">=",
            Token::LeftBrace => "{",
            Token::LeftParenthesis => "(",
            Token::LeftBracket => "[",
            Token::Less => "<",
            Token::LessEqual => "<=",
            Token::Minus => "-",
            Token::MinusEqual => "-=",
            Token::Percent => "%",
            Token::PercentEqual => "%=",
            Token::Plus => "+",
            Token::PlusEqual => "+=",
            Token::Return => "return",
            Token::RightBrace => "}",
            Token::RightParenthesis => ")",
            Token::RightBracket => "]",
            Token::Semicolon => ";",
            Token::Slash => "/",
            Token::SlashEqual => "/=",
            Token::Star => "*",
            Token::StarEqual => "*=",
        }
    }

    pub fn to_owned(&self) -> TokenOwned {
        match self {
            Token::ArrowThin => TokenOwned::ArrowThin,
            Token::Async => TokenOwned::Async,
            Token::BangEqual => TokenOwned::BangEqual,
            Token::Bang => TokenOwned::Bang,
            Token::Bool => TokenOwned::Bool,
            Token::Boolean(boolean) => TokenOwned::Boolean(boolean.to_string()),
            Token::Break => TokenOwned::Break,
            Token::Byte(byte) => TokenOwned::Byte(byte.to_string()),
            Token::Character(character) => TokenOwned::Character(*character),
            Token::Colon => TokenOwned::Colon,
            Token::Comma => TokenOwned::Comma,
            Token::Dot => TokenOwned::Dot,
            Token::DoubleAmpersand => TokenOwned::DoubleAmpersand,
            Token::DoubleDot => TokenOwned::DoubleDot,
            Token::DoubleEqual => TokenOwned::DoubleEqual,
            Token::DoublePipe => TokenOwned::DoublePipe,
            Token::Else => TokenOwned::Else,
            Token::Eof => TokenOwned::Eof,
            Token::Equal => TokenOwned::Equal,
            Token::Float(float) => TokenOwned::Float(float.to_string()),
            Token::FloatKeyword => TokenOwned::FloatKeyword,
            Token::Fn => TokenOwned::Fn,
            Token::Greater => TokenOwned::Greater,
            Token::GreaterEqual => TokenOwned::GreaterOrEqual,
            Token::Identifier(text) => TokenOwned::Identifier(text.to_string()),
            Token::If => TokenOwned::If,
            Token::Int => TokenOwned::Int,
            Token::Integer(integer) => TokenOwned::Integer(integer.to_string()),
            Token::LeftBrace => TokenOwned::LeftCurlyBrace,
            Token::LeftParenthesis => TokenOwned::LeftParenthesis,
            Token::LeftBracket => TokenOwned::LeftSquareBrace,
            Token::Let => TokenOwned::Let,
            Token::Less => TokenOwned::Less,
            Token::LessEqual => TokenOwned::LessOrEqual,
            Token::Loop => TokenOwned::Loop,
            Token::Map => TokenOwned::Map,
            Token::Minus => TokenOwned::Minus,
            Token::MinusEqual => TokenOwned::MinusEqual,
            Token::Mut => TokenOwned::Mut,
            Token::Percent => TokenOwned::Percent,
            Token::PercentEqual => TokenOwned::PercentEqual,
            Token::Plus => TokenOwned::Plus,
            Token::PlusEqual => TokenOwned::PlusEqual,
            Token::Return => TokenOwned::Return,
            Token::RightBrace => TokenOwned::RightCurlyBrace,
            Token::RightParenthesis => TokenOwned::RightParenthesis,
            Token::RightBracket => TokenOwned::RightSquareBrace,
            Token::Semicolon => TokenOwned::Semicolon,
            Token::Star => TokenOwned::Star,
            Token::StarEqual => TokenOwned::StarEqual,
            Token::Slash => TokenOwned::Slash,
            Token::SlashEqual => TokenOwned::SlashEqual,
            Token::String(text) => TokenOwned::String(text.to_string()),
            Token::Str => TokenOwned::Str,
            Token::Struct => TokenOwned::Struct,
            Token::While => TokenOwned::While,
        }
    }

    pub fn kind(&self) -> TokenKind {
        match self {
            Token::ArrowThin => TokenKind::ArrowThin,
            Token::Async => TokenKind::Async,
            Token::BangEqual => TokenKind::BangEqual,
            Token::Bang => TokenKind::Bang,
            Token::Bool => TokenKind::Bool,
            Token::Boolean(_) => TokenKind::Boolean,
            Token::Break => TokenKind::Break,
            Token::Byte(_) => TokenKind::Byte,
            Token::Character(_) => TokenKind::Character,
            Token::Colon => TokenKind::Colon,
            Token::Comma => TokenKind::Comma,
            Token::Dot => TokenKind::Dot,
            Token::DoubleAmpersand => TokenKind::DoubleAmpersand,
            Token::DoubleDot => TokenKind::DoubleDot,
            Token::DoubleEqual => TokenKind::DoubleEqual,
            Token::DoublePipe => TokenKind::DoublePipe,
            Token::Else => TokenKind::Else,
            Token::Eof => TokenKind::Eof,
            Token::Equal => TokenKind::Equal,
            Token::Float(_) => TokenKind::Float,
            Token::FloatKeyword => TokenKind::FloatKeyword,
            Token::Fn => TokenKind::Fn,
            Token::Greater => TokenKind::Greater,
            Token::GreaterEqual => TokenKind::GreaterEqual,
            Token::Identifier(_) => TokenKind::Identifier,
            Token::If => TokenKind::If,
            Token::Int => TokenKind::Int,
            Token::Integer(_) => TokenKind::Integer,
            Token::LeftBrace => TokenKind::LeftBrace,
            Token::LeftParenthesis => TokenKind::LeftParenthesis,
            Token::LeftBracket => TokenKind::LeftBracket,
            Token::Let => TokenKind::Let,
            Token::Less => TokenKind::Less,
            Token::LessEqual => TokenKind::LessEqual,
            Token::Loop => TokenKind::Loop,
            Token::Map => TokenKind::Map,
            Token::Minus => TokenKind::Minus,
            Token::MinusEqual => TokenKind::MinusEqual,
            Token::Mut => TokenKind::Mut,
            Token::Percent => TokenKind::Percent,
            Token::PercentEqual => TokenKind::PercentEqual,
            Token::Plus => TokenKind::Plus,
            Token::PlusEqual => TokenKind::PlusEqual,
            Token::Return => TokenKind::Return,
            Token::RightBrace => TokenKind::RightBrace,
            Token::RightParenthesis => TokenKind::RightParenthesis,
            Token::RightBracket => TokenKind::RightBracket,
            Token::Semicolon => TokenKind::Semicolon,
            Token::Star => TokenKind::Star,
            Token::StarEqual => TokenKind::StarEqual,
            Token::Slash => TokenKind::Slash,
            Token::SlashEqual => TokenKind::SlashEqual,
            Token::Str => TokenKind::Str,
            Token::String(_) => TokenKind::String,
            Token::Struct => TokenKind::Struct,
            Token::While => TokenKind::While,
        }
    }

    /// Returns true if the token yields a value, begins an expression or is an expression operator.
    pub fn is_expression(&self) -> bool {
        matches!(
            self,
            Token::Boolean(_)
                | Token::Byte(_)
                | Token::Character(_)
                | Token::Float(_)
                | Token::Identifier(_)
                | Token::Integer(_)
                | Token::String(_)
                | Token::Break
                | Token::If
                | Token::Return
                | Token::Map
                | Token::Loop
                | Token::Struct
                | Token::BangEqual
                | Token::DoubleAmpersand
                | Token::DoubleEqual
                | Token::DoublePipe
                | Token::Equal
                | Token::Greater
                | Token::GreaterEqual
                | Token::LeftBrace
                | Token::LeftParenthesis
                | Token::LeftBracket
                | Token::Less
                | Token::LessEqual
                | Token::Minus
                | Token::MinusEqual
                | Token::Percent
                | Token::PercentEqual
                | Token::Plus
                | Token::PlusEqual
                | Token::Slash
                | Token::SlashEqual
                | Token::Star
                | Token::StarEqual
        )
    }
}

impl<'src> Display for Token<'src> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Token::ArrowThin => write!(f, "->"),
            Token::Async => write!(f, "async"),
            Token::BangEqual => write!(f, "!="),
            Token::Bang => write!(f, "!"),
            Token::Bool => write!(f, "bool"),
            Token::Boolean(value) => write!(f, "{value}"),
            Token::Break => write!(f, "break"),
            Token::Byte(value) => write!(f, "{value}"),
            Token::Character(value) => write!(f, "{value}"),
            Token::Colon => write!(f, ":"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::DoubleAmpersand => write!(f, "&&"),
            Token::DoubleDot => write!(f, ".."),
            Token::DoubleEqual => write!(f, "=="),
            Token::DoublePipe => write!(f, "||"),
            Token::Else => write!(f, "else"),
            Token::Eof => write!(f, "EOF"),
            Token::Equal => write!(f, "="),
            Token::Float(value) => write!(f, "{value}"),
            Token::FloatKeyword => write!(f, "float"),
            Token::Fn => write!(f, "fn"),
            Token::Greater => write!(f, ">"),
            Token::GreaterEqual => write!(f, ">="),
            Token::Identifier(value) => write!(f, "{value}"),
            Token::If => write!(f, "if"),
            Token::Int => write!(f, "int"),
            Token::Integer(value) => write!(f, "{value}"),
            Token::LeftBrace => write!(f, "{{"),
            Token::LeftParenthesis => write!(f, "("),
            Token::LeftBracket => write!(f, "["),
            Token::Let => write!(f, "let"),
            Token::Less => write!(f, "<"),
            Token::LessEqual => write!(f, "<="),
            Token::Loop => write!(f, "loop"),
            Token::Map => write!(f, "map"),
            Token::Minus => write!(f, "-"),
            Token::MinusEqual => write!(f, "-="),
            Token::Mut => write!(f, "mut"),
            Token::Percent => write!(f, "%"),
            Token::PercentEqual => write!(f, "%="),
            Token::Plus => write!(f, "+"),
            Token::PlusEqual => write!(f, "+="),
            Token::Return => write!(f, "return"),
            Token::RightBrace => write!(f, "}}"),
            Token::RightParenthesis => write!(f, ")"),
            Token::RightBracket => write!(f, "]"),
            Token::Semicolon => write!(f, ";"),
            Token::Slash => write!(f, "/"),
            Token::SlashEqual => write!(f, "/="),
            Token::Star => write!(f, "*"),
            Token::StarEqual => write!(f, "*="),
            Token::Str => write!(f, "str"),
            Token::String(value) => write!(f, "{value}"),
            Token::Struct => write!(f, "struct"),
            Token::While => write!(f, "while"),
        }
    }
}

/// Owned representation of a source token.
///
/// If a [Token] borrows from the source text, its TokenOwned omits the data.
#[derive(Debug, PartialEq, Clone)]
pub enum TokenOwned {
    Eof,

    Identifier(String),

    // Hard-coded values
    Boolean(String),
    Byte(String),
    Character(char),
    Float(String),
    Integer(String),
    String(String),

    // Keywords
    Async,
    Bool,
    Break,
    Else,
    FloatKeyword,
    Fn,
    If,
    Int,
    Let,
    Loop,
    Map,
    Mut,
    Return,
    Str,
    While,

    // Symbols
    ArrowThin,
    Bang,
    BangEqual,
    Colon,
    Comma,
    Dot,
    DoubleAmpersand,
    DoubleDot,
    DoubleEqual,
    DoublePipe,
    Equal,
    Greater,
    GreaterOrEqual,
    LeftCurlyBrace,
    LeftParenthesis,
    LeftSquareBrace,
    Less,
    LessOrEqual,
    Minus,
    MinusEqual,
    Percent,
    PercentEqual,
    Plus,
    PlusEqual,
    RightCurlyBrace,
    RightParenthesis,
    RightSquareBrace,
    Semicolon,
    Star,
    StarEqual,
    Struct,
    Slash,
    SlashEqual,
}

impl Display for TokenOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TokenOwned::ArrowThin => Token::ArrowThin.fmt(f),
            TokenOwned::Async => Token::Async.fmt(f),
            TokenOwned::Bang => Token::Bang.fmt(f),
            TokenOwned::BangEqual => Token::BangEqual.fmt(f),
            TokenOwned::Bool => Token::Bool.fmt(f),
            TokenOwned::Boolean(boolean) => Token::Boolean(boolean).fmt(f),
            TokenOwned::Break => Token::Break.fmt(f),
            TokenOwned::Byte(byte) => Token::Byte(byte).fmt(f),
            TokenOwned::Character(character) => Token::Character(*character).fmt(f),
            TokenOwned::Colon => Token::Colon.fmt(f),
            TokenOwned::Comma => Token::Comma.fmt(f),
            TokenOwned::Dot => Token::Dot.fmt(f),
            TokenOwned::DoubleAmpersand => Token::DoubleAmpersand.fmt(f),
            TokenOwned::DoubleDot => Token::DoubleDot.fmt(f),
            TokenOwned::DoubleEqual => Token::DoubleEqual.fmt(f),
            TokenOwned::DoublePipe => Token::DoublePipe.fmt(f),
            TokenOwned::Else => Token::Else.fmt(f),
            TokenOwned::Eof => Token::Eof.fmt(f),
            TokenOwned::Equal => Token::Equal.fmt(f),
            TokenOwned::Float(float) => Token::Float(float).fmt(f),
            TokenOwned::FloatKeyword => Token::FloatKeyword.fmt(f),
            TokenOwned::Fn => Token::Fn.fmt(f),
            TokenOwned::Greater => Token::Greater.fmt(f),
            TokenOwned::GreaterOrEqual => Token::GreaterEqual.fmt(f),
            TokenOwned::Identifier(text) => Token::Identifier(text).fmt(f),
            TokenOwned::If => Token::If.fmt(f),
            TokenOwned::Int => Token::Int.fmt(f),
            TokenOwned::Integer(integer) => Token::Integer(integer).fmt(f),
            TokenOwned::LeftCurlyBrace => Token::LeftBrace.fmt(f),
            TokenOwned::LeftParenthesis => Token::LeftParenthesis.fmt(f),
            TokenOwned::LeftSquareBrace => Token::LeftBracket.fmt(f),
            TokenOwned::Let => Token::Let.fmt(f),
            TokenOwned::Less => Token::Less.fmt(f),
            TokenOwned::LessOrEqual => Token::LessEqual.fmt(f),
            TokenOwned::Loop => Token::Loop.fmt(f),
            TokenOwned::Map => Token::Map.fmt(f),
            TokenOwned::Minus => Token::Minus.fmt(f),
            TokenOwned::MinusEqual => Token::MinusEqual.fmt(f),
            TokenOwned::Mut => Token::Mut.fmt(f),
            TokenOwned::Percent => Token::Percent.fmt(f),
            TokenOwned::PercentEqual => Token::PercentEqual.fmt(f),
            TokenOwned::Plus => Token::Plus.fmt(f),
            TokenOwned::PlusEqual => Token::PlusEqual.fmt(f),
            TokenOwned::Return => Token::Return.fmt(f),
            TokenOwned::RightCurlyBrace => Token::RightBrace.fmt(f),
            TokenOwned::RightParenthesis => Token::RightParenthesis.fmt(f),
            TokenOwned::RightSquareBrace => Token::RightBracket.fmt(f),
            TokenOwned::Semicolon => Token::Semicolon.fmt(f),
            TokenOwned::Star => Token::Star.fmt(f),
            TokenOwned::StarEqual => Token::StarEqual.fmt(f),
            TokenOwned::Slash => Token::Slash.fmt(f),
            TokenOwned::SlashEqual => Token::SlashEqual.fmt(f),
            TokenOwned::Str => Token::Str.fmt(f),
            TokenOwned::String(string) => Token::String(string).fmt(f),
            TokenOwned::Struct => Token::Struct.fmt(f),
            TokenOwned::While => Token::While.fmt(f),
        }
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
