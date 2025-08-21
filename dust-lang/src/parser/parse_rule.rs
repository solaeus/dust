use std::fmt::{self, Display, Formatter};

use crate::{
    Token,
    parser::{ParseError, Parser},
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

impl From<Token> for ParseRule<'_> {
    fn from(token: Token) -> Self {
        match token {
            Token::Any => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            Token::ArrowThin => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Asterisk => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::PrimaryMath,
            },
            Token::Async => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Bang => ParseRule {
                prefix: Some(Parser::parse_unary),
                infix: None,
                precedence: Precedence::Unary,
            },
            Token::BangEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            Token::Bool => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Boolean => ParseRule {
                prefix: Some(Parser::parse_boolean),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Break => todo!(),
            Token::Byte => ParseRule {
                prefix: Some(Parser::parse_byte),
                infix: None,
                precedence: Precedence::None,
            },
            Token::ByteKeyword => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Cell => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Character => ParseRule {
                prefix: Some(Parser::parse_character),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Colon => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Comma => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::Const => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::Assignment,
            },
            Token::Dot => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            Token::DoubleAmpersand => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_logical_binary),
                precedence: Precedence::Logic,
            },
            Token::DoubleEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            Token::DoublePipe => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_logical_binary),
                precedence: Precedence::Logic,
            },
            Token::DoubleDot => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Eof => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::Equal => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::Assignment,
            },
            Token::Else => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::Float => ParseRule {
                prefix: Some(Parser::parse_float),
                infix: None,
                precedence: Precedence::None,
            },
            Token::FloatKeyword => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Fn => ParseRule {
                prefix: Some(Parser::parse_function),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Greater => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            Token::GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            Token::Identifier => ParseRule {
                prefix: Some(Parser::parse_identifier),
                infix: None,
                precedence: Precedence::None,
            },
            Token::If => ParseRule {
                prefix: Some(Parser::parse_if),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Int => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Integer => ParseRule {
                prefix: Some(Parser::parse_integer_expression),
                infix: None,
                precedence: Precedence::None,
            },
            Token::LeftBrace => ParseRule {
                prefix: Some(Parser::parse_block),
                infix: None,
                precedence: Precedence::None,
            },
            Token::LeftParenthesis => ParseRule {
                prefix: Some(Parser::parse_grouped),
                infix: Some(Parser::parse_call),
                precedence: Precedence::Call,
            },
            Token::LeftBracket => ParseRule {
                prefix: Some(Parser::parse_array),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Less => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            Token::LessEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_comparison_binary),
                precedence: Precedence::Comparison,
            },
            Token::Let => ParseRule {
                prefix: Some(Parser::parse_let_statement),
                infix: None,
                precedence: Precedence::Assignment,
            },
            Token::List => ParseRule {
                prefix: Some(Parser::parse_list),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Loop => todo!(),
            Token::Map => todo!(),
            Token::Minus => ParseRule {
                prefix: Some(Parser::parse_unary),
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::SecondaryMath,
            },
            Token::MinusEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Assignment,
            },
            Token::Mod => ParseRule {
                prefix: Some(Parser::parse_mod),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Mut => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::Percent => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::PrimaryMath,
            },
            Token::PercentEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Assignment,
            },
            Token::Plus => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::SecondaryMath,
            },
            Token::PlusEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Assignment,
            },
            Token::Return => ParseRule {
                prefix: Some(Parser::parse_return),
                infix: None,
                precedence: Precedence::None,
            },
            Token::RightBrace => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            Token::RightParenthesis => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::RightBracket => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            Token::Semicolon => ParseRule {
                prefix: Some(Parser::parse_semicolon),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Slash => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::PrimaryMath,
            },
            Token::SlashEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Assignment,
            },
            Token::StarEqual => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_math_binary),
                precedence: Precedence::Assignment,
            },
            Token::Str => ParseRule {
                prefix: Some(Parser::parse_str),
                infix: None,
                precedence: Precedence::None,
            },
            Token::String => ParseRule {
                prefix: Some(Parser::parse_string_expression),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Struct => ParseRule {
                prefix: Some(Parser::parse_unexpected),
                infix: None,
                precedence: Precedence::None,
            },
            Token::Use => ParseRule {
                prefix: Some(Parser::parse_use),
                infix: None,
                precedence: Precedence::None,
            },
            Token::While => ParseRule {
                prefix: Some(Parser::parse_while),
                infix: None,
                precedence: Precedence::None,
            },
        }
    }
}

/// Operator precedence levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    Primary = 8,
    Call = 7,
    Unary = 6,
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
            Precedence::PrimaryMath => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
        }
    }
}

impl Display for Precedence {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
