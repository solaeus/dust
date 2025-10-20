use std::fmt::{self, Display, Formatter};

use crate::{
    parser::{ParseError, Parser},
    token::TokenKind,
};

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
}

impl From<TokenKind> for ParseRule<'_> {
    fn from(token: TokenKind) -> Self {
        match token {
            TokenKind::Any => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::ArrowThin => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Asterisk => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::PrimaryMath,
            },
            TokenKind::AsteriskEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::Assignment,
            },
            TokenKind::Async => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Bang => ParseRule {
                prefix: Some(Parser::parse_unary_expression),
                infix: None,
                precedence: Precedence::Unary,
            },
            TokenKind::BangEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::Comparison,
            },
            TokenKind::BlockComment => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Bool => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::FalseValue => ParseRule {
                prefix: Some(Parser::parse_boolean_expression),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::TrueValue => ParseRule {
                prefix: Some(Parser::parse_boolean_expression),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Break => ParseRule {
                prefix: Some(Parser::parse_break_expression),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::ByteValue => ParseRule {
                prefix: Some(Parser::parse_byte_expression),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Byte => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Caret => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::Exponent,
            },
            TokenKind::Cell => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Char => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::CharacterValue => ParseRule {
                prefix: Some(Parser::parse_character_expression),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Colon => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Comma => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Const => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::Assignment,
            },
            TokenKind::Dot => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::DoubleAmpersand => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::Logic,
            },
            TokenKind::DoubleColon => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_path),
                precedence: Precedence::Path,
            },
            TokenKind::DoubleEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::Comparison,
            },
            TokenKind::DoublePipe => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::Logic,
            },
            TokenKind::DoubleDot => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Eof => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Equal => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_reassign_statement),
                precedence: Precedence::Assignment,
            },
            TokenKind::Else => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::FloatValue => ParseRule {
                prefix: Some(Parser::parse_float_expression),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Float => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Fn => ParseRule {
                prefix: Some(Parser::parse_function_item_or_expression),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Greater => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::Comparison,
            },
            TokenKind::GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::Comparison,
            },
            TokenKind::Identifier => ParseRule {
                prefix: Some(Parser::parse_path_expression),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::If => ParseRule {
                prefix: Some(Parser::parse_if),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::InnerBlockDocComment => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::InnerLineDocComment => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Int => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::IntegerValue => ParseRule {
                prefix: Some(Parser::parse_integer_expression),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::LeftCurlyBrace => ParseRule {
                prefix: Some(Parser::parse_block_expression),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::LeftParenthesis => ParseRule {
                prefix: Some(Parser::parse_grouped_expression),
                infix: Some(Parser::parse_call_expression),
                precedence: Precedence::CallOrIndex,
            },
            TokenKind::LeftSquareBracket => ParseRule {
                prefix: Some(Parser::parse_list_expression),
                infix: Some(Parser::parse_index_expression),
                precedence: Precedence::CallOrIndex,
            },
            TokenKind::Less => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::Comparison,
            },
            TokenKind::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::Comparison,
            },
            TokenKind::Let => ParseRule {
                prefix: Some(Parser::parse_let_statement),
                infix: None,
                precedence: Precedence::Assignment,
            },
            TokenKind::LineComment => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Loop => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Map => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Minus => ParseRule {
                prefix: Some(Parser::parse_unary_expression),
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::SecondaryMath,
            },
            TokenKind::MinusEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::Assignment,
            },
            TokenKind::Mod => ParseRule {
                prefix: Some(Parser::parse_module_item),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Mut => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::OuterBlockDocComment => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::OuterLineDocComment => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Percent => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::PrimaryMath,
            },
            TokenKind::PercentEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::Assignment,
            },
            TokenKind::Plus => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::SecondaryMath,
            },
            TokenKind::PlusEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::Assignment,
            },
            TokenKind::Pub => ParseRule {
                prefix: Some(Parser::parse_pub_item),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Return => ParseRule {
                prefix: Some(Parser::parse_return),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::RightCurlyBrace => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::RightParenthesis => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::RightSquareBracket => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Semicolon => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_semicolon),
                precedence: Precedence::None,
            },
            TokenKind::Slash => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::PrimaryMath,
            },
            TokenKind::SlashEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary_expression),
                precedence: Precedence::Assignment,
            },
            TokenKind::Str => ParseRule {
                prefix: Some(Parser::parse_str),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::StringValue => ParseRule {
                prefix: Some(Parser::parse_string_expression),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Struct => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Unknown => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Use => ParseRule {
                prefix: Some(Parser::parse_use_item),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::While => ParseRule {
                prefix: Some(Parser::parse_while_expression),
                infix: None,
                precedence: Precedence::None,
            },
        }
    }
}

/// Operator precedence levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    Primary = 10,
    Path = 9,
    CallOrIndex = 8,
    Unary = 7,
    Exponent = 6,
    PrimaryMath = 5,
    SecondaryMath = 4,
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
            Precedence::Comparison => Precedence::SecondaryMath,
            Precedence::SecondaryMath => Precedence::PrimaryMath,
            Precedence::PrimaryMath => Precedence::Exponent,
            Precedence::Exponent => Precedence::Unary,
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
