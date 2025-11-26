use std::fmt::{self, Display, Formatter};

use crate::{
    parser::{ParseError, Parser},
    token::TokenKind,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Associativity {
    Left,
    Right,
}

pub type ParseLogic<'a> = fn(&mut Parser<'a>) -> Result<(), ParseError>;

/// Pratt parsing rule for a token in the Dust language.
///
/// Each token can have a prefix and/or infix parsing function associated with it, which is used to
/// parse expressions involving that token. The precedence indicates how the token should be treated
/// for operator precedence during parsing.
///
/// See `Parser::pratt`, `Parser::parse_expression`, and `Parser::parse_sub_expression` to see the actual
/// use of precedence.
#[derive(Debug, Clone, Copy)]
pub struct ParseRule<'a> {
    pub prefix: Option<ParseLogic<'a>>,
    pub infix: Option<ParseLogic<'a>>,
    pub precedence: Precedence,
    pub associativity: Associativity,
}

impl From<TokenKind> for ParseRule<'_> {
    fn from(token: TokenKind) -> Self {
        match token {
            TokenKind::Any => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::ArrowThin => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::As => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_as_expression),
                precedence: Precedence::PrimaryMath,
                associativity: Associativity::Left,
            },
            TokenKind::Asterisk => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::PrimaryMath,
                associativity: Associativity::Left,
            },
            TokenKind::AsteriskEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::Assignment,
                associativity: Associativity::Right,
            },
            TokenKind::Async => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Bang => ParseRule {
                prefix: Some(Parser::parse_unary_expression),
                infix: None,
                precedence: Precedence::Unary,
                associativity: Associativity::Left,
            },
            TokenKind::BangEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::Comparison,
                associativity: Associativity::Left,
            },
            TokenKind::BlockComment => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Bool => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::FalseValue => ParseRule {
                prefix: Some(Parser::parse_boolean_expression),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::TrueValue => ParseRule {
                prefix: Some(Parser::parse_boolean_expression),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Break => ParseRule {
                prefix: Some(Parser::parse_break_expression),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::ByteValue => ParseRule {
                prefix: Some(Parser::parse_byte_expression),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Byte => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Caret => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::Exponent,
                associativity: Associativity::Right,
            },
            TokenKind::CaretEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::Assignment,
                associativity: Associativity::Right,
            },
            TokenKind::Cell => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Char => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::CharacterValue => ParseRule {
                prefix: Some(Parser::parse_character_expression),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Colon => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Comma => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Const => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::Assignment,
                associativity: Associativity::Left,
            },
            TokenKind::Dot => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::DoubleAmpersand => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::Logic,
                associativity: Associativity::Left,
            },
            TokenKind::DoubleColon => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_path),
                precedence: Precedence::Path,
                associativity: Associativity::Left,
            },
            TokenKind::DoubleEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::Comparison,
                associativity: Associativity::Left,
            },
            TokenKind::DoublePipe => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::Logic,
                associativity: Associativity::Left,
            },
            TokenKind::DoubleDot => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Eof => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Equal => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_reassignment_statement),
                precedence: Precedence::Assignment,
                associativity: Associativity::Right,
            },
            TokenKind::Else => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::FloatValue => ParseRule {
                prefix: Some(Parser::parse_float_expression),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Float => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Fn => ParseRule {
                prefix: Some(Parser::parse_function_item_or_expression),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Greater => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::Comparison,
                associativity: Associativity::Left,
            },
            TokenKind::GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::Comparison,
                associativity: Associativity::Left,
            },
            TokenKind::Identifier => ParseRule {
                prefix: Some(Parser::parse_path_expression),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::If => ParseRule {
                prefix: Some(Parser::parse_if_expression),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::InnerBlockDocComment => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::InnerLineDocComment => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Int => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::IntegerValue => ParseRule {
                prefix: Some(Parser::parse_integer_expression),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::LeftCurlyBrace => ParseRule {
                prefix: Some(Parser::parse_block_expression),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::LeftParenthesis => ParseRule {
                prefix: Some(Parser::parse_grouped_expression),
                infix: Some(Parser::parse_call_expression),
                precedence: Precedence::CallOrIndex,
                associativity: Associativity::Left,
            },
            TokenKind::LeftSquareBracket => ParseRule {
                prefix: Some(Parser::parse_list_expression),
                infix: Some(Parser::parse_index_expression),
                precedence: Precedence::CallOrIndex,
                associativity: Associativity::Left,
            },
            TokenKind::Less => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::Comparison,
                associativity: Associativity::Left,
            },
            TokenKind::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::Comparison,
                associativity: Associativity::Left,
            },
            TokenKind::Let => ParseRule {
                prefix: Some(Parser::parse_let_statement),
                infix: None,
                precedence: Precedence::Assignment,
                associativity: Associativity::Left,
            },
            TokenKind::LineComment => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Loop => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Map => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Minus => ParseRule {
                prefix: Some(Parser::parse_unary_expression),
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::SecondaryMath,
                associativity: Associativity::Left,
            },
            TokenKind::MinusEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::Assignment,
                associativity: Associativity::Right,
            },
            TokenKind::Mod => ParseRule {
                prefix: Some(Parser::parse_module_item),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Mut => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::OuterBlockDocComment => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::OuterLineDocComment => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Percent => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::PrimaryMath,
                associativity: Associativity::Left,
            },
            TokenKind::PercentEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::Assignment,
                associativity: Associativity::Right,
            },
            TokenKind::Plus => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::SecondaryMath,
                associativity: Associativity::Left,
            },
            TokenKind::PlusEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::Assignment,
                associativity: Associativity::Right,
            },
            TokenKind::Pub => ParseRule {
                prefix: Some(Parser::parse_pub_item),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Return => ParseRule {
                prefix: Some(Parser::parse_return),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::RightCurlyBrace => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::RightParenthesis => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::RightSquareBracket => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Semicolon => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_semicolon),
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Slash => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::PrimaryMath,
                associativity: Associativity::Left,
            },
            TokenKind::SlashEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_operator),
                precedence: Precedence::Assignment,
                associativity: Associativity::Right,
            },
            TokenKind::Str => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::StringValue => ParseRule {
                prefix: Some(Parser::parse_string_expression),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Struct => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Unknown => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Use => ParseRule {
                prefix: Some(Parser::parse_use_item),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::While => ParseRule {
                prefix: Some(Parser::parse_while_expression),
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
        }
    }
}

/// Operator precedence levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    Primary = 11,
    Path = 10,
    CallOrIndex = 9,
    Unary = 8,
    AsKeyword = 7,
    PrimaryMath = 6,
    SecondaryMath = 5,
    Exponent = 4,
    Comparison = 3,
    Logic = 2,
    Assignment = 1,
    None = 0,
}

impl Precedence {
    pub fn increment(&self) -> Self {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Logic,
            Precedence::Logic => Precedence::Comparison,
            Precedence::Comparison => Precedence::Exponent,
            Precedence::Exponent => Precedence::SecondaryMath,
            Precedence::SecondaryMath => Precedence::PrimaryMath,
            Precedence::PrimaryMath => Precedence::AsKeyword,
            Precedence::AsKeyword => Precedence::Unary,
            Precedence::Unary => Precedence::CallOrIndex,
            Precedence::CallOrIndex => Precedence::Path,
            Precedence::Path => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
        }
    }
}

impl Display for Precedence {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
