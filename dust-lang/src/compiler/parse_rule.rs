use std::fmt::{self, Display, Formatter};

use super::{CompileError, Compiler};

pub type Parser<'a> = fn(&mut Compiler<'a>) -> Result<(), CompileError>;

/// Rule that defines how to parse a token.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParseRule<'a> {
    pub prefix: Option<Parser<'a>>,
    pub infix: Option<Parser<'a>>,
    pub precedence: Precedence,
}

/// Operator precedence levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    Primary = 8,
    Call = 7,
    Unary = 6,
    Factor = 5,
    Term = 4,
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
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
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

pub fn create_look_up_table<'a>() -> [ParseRule<'a>; 55] {
    [
        (ParseRule {
            prefix: Some(Compiler::parse_boolean),
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: Some(Compiler::parse_byte),
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: Some(Compiler::parse_character),
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: Some(Compiler::parse_float),
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: Some(Compiler::parse_variable),
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: Some(Compiler::parse_integer),
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: Some(Compiler::parse_string),
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: Some(Compiler::parse_function),
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: Some(Compiler::parse_if),
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: Some(Compiler::parse_let),
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: Some(Compiler::parse_return),
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: Some(Compiler::parse_while),
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_comparison_binary),
            precedence: Precedence::Comparison,
        }),
        (ParseRule {
            prefix: Some(Compiler::parse_unary),
            infix: None,
            precedence: Precedence::Unary,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_logical_binary),
            precedence: Precedence::Logic,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_comparison_binary),
            precedence: Precedence::Comparison,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_logical_binary),
            precedence: Precedence::Logic,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_comparison_binary),
            precedence: Precedence::Comparison,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_comparison_binary),
            precedence: Precedence::Comparison,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_comparison_binary),
            precedence: Precedence::Comparison,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: Some(Compiler::parse_grouped),
            infix: Some(Compiler::parse_call),
            precedence: Precedence::Call,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_comparison_binary),
            precedence: Precedence::Comparison,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_comparison_binary),
            precedence: Precedence::Comparison,
        }),
        (ParseRule {
            prefix: Some(Compiler::parse_unary),
            infix: Some(Compiler::parse_math_binary),
            precedence: Precedence::Term,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_comparison_binary),
            precedence: Precedence::Comparison,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_math_binary),
            precedence: Precedence::Factor,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_math_binary),
            precedence: Precedence::Assignment,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_math_binary),
            precedence: Precedence::Term,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_math_binary),
            precedence: Precedence::Assignment,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_math_binary),
            precedence: Precedence::Factor,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_math_binary),
            precedence: Precedence::Assignment,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_math_binary),
            precedence: Precedence::Factor,
        }),
        (ParseRule {
            prefix: None,
            infix: Some(Compiler::parse_math_binary),
            precedence: Precedence::Assignment,
        }),
        (ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }),
    ]
}

#[cfg(test)]
mod tests {
    use crate::TokenKind;

    use enum_iterator::all;

    use super::*;

    #[test]
    fn tokens_correspond_to_the_correct_rule() {
        let rule_table = create_look_up_table();
        let actual_pairs = all::<TokenKind>().zip(rule_table);
        let expected_pairs = [
            (
                TokenKind::Boolean,
                ParseRule {
                    prefix: Some(Compiler::parse_boolean),
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Byte,
                ParseRule {
                    prefix: Some(Compiler::parse_byte),
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Character,
                ParseRule {
                    prefix: Some(Compiler::parse_character),
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Float,
                ParseRule {
                    prefix: Some(Compiler::parse_float),
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Identifier,
                ParseRule {
                    prefix: Some(Compiler::parse_variable),
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Integer,
                ParseRule {
                    prefix: Some(Compiler::parse_integer),
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::String,
                ParseRule {
                    prefix: Some(Compiler::parse_string),
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Bool,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Break,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Else,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::FloatKeyword,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Fn,
                ParseRule {
                    prefix: Some(Compiler::parse_function),
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::If,
                ParseRule {
                    prefix: Some(Compiler::parse_if),
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Int,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Let,
                ParseRule {
                    prefix: Some(Compiler::parse_let),
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Loop,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Map,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Mut,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Return,
                ParseRule {
                    prefix: Some(Compiler::parse_return),
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Str,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Struct,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::While,
                ParseRule {
                    prefix: Some(Compiler::parse_while),
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::ArrowThin,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::BangEqual,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_comparison_binary),
                    precedence: Precedence::Comparison,
                },
            ),
            (
                TokenKind::Bang,
                ParseRule {
                    prefix: Some(Compiler::parse_unary),
                    infix: None,
                    precedence: Precedence::Unary,
                },
            ),
            (
                TokenKind::Colon,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Comma,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Dot,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::DoubleAmpersand,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_logical_binary),
                    precedence: Precedence::Logic,
                },
            ),
            (
                TokenKind::DoubleDot,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::DoubleEqual,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_comparison_binary),
                    precedence: Precedence::Comparison,
                },
            ),
            (
                TokenKind::DoublePipe,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_logical_binary),
                    precedence: Precedence::Logic,
                },
            ),
            (
                TokenKind::Equal,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_comparison_binary),
                    precedence: Precedence::Comparison,
                },
            ),
            (
                TokenKind::Greater,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_comparison_binary),
                    precedence: Precedence::Comparison,
                },
            ),
            (
                TokenKind::GreaterEqual,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_comparison_binary),
                    precedence: Precedence::Comparison,
                },
            ),
            (
                TokenKind::LeftBrace,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::LeftBracket,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::LeftParenthesis,
                ParseRule {
                    prefix: Some(Compiler::parse_grouped),
                    infix: Some(Compiler::parse_call),
                    precedence: Precedence::Call,
                },
            ),
            (
                TokenKind::Less,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_comparison_binary),
                    precedence: Precedence::Comparison,
                },
            ),
            (
                TokenKind::LessEqual,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_comparison_binary),
                    precedence: Precedence::Comparison,
                },
            ),
            (
                TokenKind::Minus,
                ParseRule {
                    prefix: Some(Compiler::parse_unary),
                    infix: Some(Compiler::parse_math_binary),
                    precedence: Precedence::Term,
                },
            ),
            (
                TokenKind::MinusEqual,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_comparison_binary),
                    precedence: Precedence::Comparison,
                },
            ),
            (
                TokenKind::Percent,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_math_binary),
                    precedence: Precedence::Factor,
                },
            ),
            (
                TokenKind::PercentEqual,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_math_binary),
                    precedence: Precedence::Assignment,
                },
            ),
            (
                TokenKind::Plus,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_math_binary),
                    precedence: Precedence::Term,
                },
            ),
            (
                TokenKind::PlusEqual,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_math_binary),
                    precedence: Precedence::Assignment,
                },
            ),
            (
                TokenKind::RightBrace,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::RightBracket,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::RightParenthesis,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Semicolon,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
            (
                TokenKind::Slash,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_math_binary),
                    precedence: Precedence::Factor,
                },
            ),
            (
                TokenKind::SlashEqual,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_math_binary),
                    precedence: Precedence::Assignment,
                },
            ),
            (
                TokenKind::Star,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_math_binary),
                    precedence: Precedence::Factor,
                },
            ),
            (
                TokenKind::StarEqual,
                ParseRule {
                    prefix: None,
                    infix: Some(Compiler::parse_math_binary),
                    precedence: Precedence::Assignment,
                },
            ),
            (
                TokenKind::Eof,
                ParseRule {
                    prefix: None,
                    infix: None,
                    precedence: Precedence::None,
                },
            ),
        ];

        for (index, (actual_pair, expected_pair)) in
            actual_pairs.zip(expected_pairs.into_iter()).enumerate()
        {
            assert_eq!(actual_pair, expected_pair, "Index {index}");
        }

        assert_eq!(all::<TokenKind>().count(), expected_pairs.len());
    }
}
