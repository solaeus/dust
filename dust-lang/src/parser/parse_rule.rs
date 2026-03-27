use crate::{
    parser::{Parser, error::ParseError},
    syntax::node::SyntaxNode,
    token::TokenKind,
};

pub type PrefixParser<'a> = fn(&mut Parser<'a>) -> Result<SyntaxNode, ParseError>;
pub type InfixParser<'a> = fn(&mut Parser<'a>, SyntaxNode) -> Result<SyntaxNode, ParseError>;

/// Pratt parsing rule for a single token.
///
/// Each token can have a prefix and/or infix parsing function associated with it, which is used to
/// parse an item, statement or expression involving that token. The [`Precedence`][] determines the
/// order of operations when parsing infix operators and the [`Associativity`][] determines how
/// operators of the same precedence are grouped.
#[derive(Debug)]
pub struct ParseRule<'a> {
    pub prefix: PrefixParser<'a>,
    pub infix: Option<InfixParser<'a>>,
    pub precedence: Precedence,
    pub associativity: Associativity,
}

impl From<TokenKind> for ParseRule<'_> {
    fn from(token: TokenKind) -> Self {
        match token {
            TokenKind::ArrowThin => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::As => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_as_keyword),
                precedence: Precedence::PrimaryMath,
                associativity: Associativity::Left,
            },
            TokenKind::Asterisk => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::PrimaryMath,
                associativity: Associativity::Left,
            },
            TokenKind::AsteriskEqual => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::Assignment,
                associativity: Associativity::Right,
            },
            TokenKind::Async => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Bang => ParseRule {
                prefix: Parser::parse_prefix_unary_operator,
                infix: None,
                precedence: Precedence::Unary,
                associativity: Associativity::Left,
            },
            TokenKind::BangEqual => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::Comparison,
                associativity: Associativity::Left,
            },
            TokenKind::BlockComment => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Bool => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::False => ParseRule {
                prefix: Parser::parse_prefix_boolean,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::True => ParseRule {
                prefix: Parser::parse_prefix_boolean,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Break => ParseRule {
                prefix: Parser::parse_prefix_break_keyword,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::HexIntegerLiteral => ParseRule {
                prefix: Parser::parse_prefix_hexadecimal_integer,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Caret => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::Exponent,
                associativity: Associativity::Right,
            },
            TokenKind::CaretEqual => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::Assignment,
                associativity: Associativity::Right,
            },
            TokenKind::Cell => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Char => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::CharacterLiteral => ParseRule {
                prefix: Parser::parse_prefix_character,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Colon => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Comma => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Const => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::Assignment,
                associativity: Associativity::Left,
            },
            TokenKind::Dot => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::DoubleAmpersand => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::Logic,
                associativity: Associativity::Left,
            },
            TokenKind::DoubleColon => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::DoubleEqual => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::Comparison,
                associativity: Associativity::Left,
            },
            TokenKind::DoublePipe => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::Logic,
                associativity: Associativity::Left,
            },
            TokenKind::DoubleDot => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Eof => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Equal => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_assignment_operator),
                precedence: Precedence::Assignment,
                associativity: Associativity::Right,
            },
            TokenKind::Else => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Enum => ParseRule {
                prefix: Parser::parse_prefix_enum_keyword,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::F32 => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::F64 => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::FloatLiteral => ParseRule {
                prefix: Parser::parse_prefix_float,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Fn => ParseRule {
                prefix: Parser::parse_prefix_fn_keyword,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Greater => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::Comparison,
                associativity: Associativity::Left,
            },
            TokenKind::GreaterEqual => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::Comparison,
                associativity: Associativity::Left,
            },
            TokenKind::I8 => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::I16 => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::I32 => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::I64 => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::I128 => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Identifier => ParseRule {
                prefix: Parser::parse_prefix_identifier,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::If => ParseRule {
                prefix: Parser::parse_prefix_if_keyword,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::InnerBlockDocComment => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::InnerLineDocComment => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::IntegerLiteral => ParseRule {
                prefix: Parser::parse_prefix_integer,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::LeftCurlyBrace => ParseRule {
                prefix: Parser::parse_prefix_left_brace,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::LeftParenthesis => ParseRule {
                prefix: Parser::parse_prefix_left_parenthesis,
                infix: Some(Parser::parse_infix_left_parenthesis),
                precedence: Precedence::CallOrIndex,
                associativity: Associativity::Left,
            },
            TokenKind::LeftSquareBracket => ParseRule {
                prefix: Parser::parse_prefix_left_bracket,
                infix: Some(Parser::parse_infix_left_bracket),
                precedence: Precedence::CallOrIndex,
                associativity: Associativity::Left,
            },
            TokenKind::Less => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::Comparison,
                associativity: Associativity::Left,
            },
            TokenKind::LessEqual => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::Comparison,
                associativity: Associativity::Left,
            },
            TokenKind::Let => ParseRule {
                prefix: Parser::parse_prefix_let_keyword,
                infix: None,
                precedence: Precedence::Assignment,
                associativity: Associativity::Left,
            },
            TokenKind::LineComment => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Loop => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Map => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Minus => ParseRule {
                prefix: Parser::parse_prefix_unary_operator,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::SecondaryMath,
                associativity: Associativity::Left,
            },
            TokenKind::MinusEqual => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::Assignment,
                associativity: Associativity::Right,
            },
            TokenKind::Mod => ParseRule {
                prefix: Parser::parse_prefix_mod_keyword,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Mut => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::OuterBlockDocComment => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::OuterLineDocComment => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Percent => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::PrimaryMath,
                associativity: Associativity::Left,
            },
            TokenKind::PercentEqual => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::Assignment,
                associativity: Associativity::Right,
            },
            TokenKind::Plus => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::SecondaryMath,
                associativity: Associativity::Left,
            },
            TokenKind::PlusEqual => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::Assignment,
                associativity: Associativity::Right,
            },
            TokenKind::Pub => ParseRule {
                prefix: Parser::parse_prefix_pub_keyword,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Return => ParseRule {
                prefix: Parser::parse_prefix_return_keyord,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::RightCurlyBrace => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::RightParenthesis => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::RightSquareBracket => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Semicolon => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_semicolon),
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Slash => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::PrimaryMath,
                associativity: Associativity::Left,
            },
            TokenKind::SlashEqual => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: Some(Parser::parse_infix_binary_operator),
                precedence: Precedence::Assignment,
                associativity: Associativity::Right,
            },
            TokenKind::Str => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::StringLiteral => ParseRule {
                prefix: Parser::parse_prefix_string,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Struct => ParseRule {
                prefix: Parser::parse_prefix_struct_keyword,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::U8 => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::U16 => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::U32 => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::U64 => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::U128 => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Unknown => ParseRule {
                prefix: Parser::parse_unexpected,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::Use => ParseRule {
                prefix: Parser::parse_prefix_use_keyword,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
            TokenKind::While => ParseRule {
                prefix: Parser::parse_prefix_while_keyword,
                infix: None,
                precedence: Precedence::None,
                associativity: Associativity::Left,
            },
        }
    }
}

#[derive(Debug)]
pub enum Associativity {
    Left,
    Right,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
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
