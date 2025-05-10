use crate::{Span, Token};

use super::{LexError, Lexer};

type LexAction<'src> = fn(&mut Lexer<'src>) -> Result<(Token<'src>, Span), LexError>;

pub struct LexRule<'src> {
    pub lex_action: LexAction<'src>,
}

impl From<&char> for LexRule<'_> {
    fn from(char: &char) -> Self {
        match char {
            '0'..='9' => LexRule {
                lex_action: Lexer::lex_numeric,
            },
            char if char.is_alphabetic() => LexRule {
                lex_action: Lexer::lex_keyword_or_identifier,
            },
            '"' => LexRule {
                lex_action: Lexer::lex_string,
            },
            '\'' => LexRule {
                lex_action: Lexer::lex_char,
            },
            '+' => LexRule {
                lex_action: Lexer::lex_plus,
            },
            '-' => LexRule {
                lex_action: Lexer::lex_minus,
            },
            '*' => LexRule {
                lex_action: Lexer::lex_star,
            },
            '/' => LexRule {
                lex_action: Lexer::lex_slash,
            },
            '%' => LexRule {
                lex_action: Lexer::lex_percent,
            },
            '!' => LexRule {
                lex_action: Lexer::lex_exclamation_mark,
            },
            '=' => LexRule {
                lex_action: Lexer::lex_equal,
            },
            '<' => LexRule {
                lex_action: Lexer::lex_less_than,
            },
            '>' => LexRule {
                lex_action: Lexer::lex_greater_than,
            },
            '&' => LexRule {
                lex_action: Lexer::lex_ampersand,
            },
            '|' => LexRule {
                lex_action: Lexer::lex_pipe,
            },
            '(' => LexRule {
                lex_action: Lexer::lex_left_parenthesis,
            },
            ')' => LexRule {
                lex_action: Lexer::lex_right_parenthesis,
            },
            '[' => LexRule {
                lex_action: Lexer::lex_left_bracket,
            },
            ']' => LexRule {
                lex_action: Lexer::lex_right_bracket,
            },
            '{' => LexRule {
                lex_action: Lexer::lex_left_brace,
            },
            '}' => LexRule {
                lex_action: Lexer::lex_right_brace,
            },
            ';' => LexRule {
                lex_action: Lexer::lex_semicolon,
            },
            ':' => LexRule {
                lex_action: Lexer::lex_colon,
            },
            ',' => LexRule {
                lex_action: Lexer::lex_comma,
            },
            '.' => LexRule {
                lex_action: Lexer::lex_dot,
            },
            _ => LexRule {
                lex_action: Lexer::lex_unexpected,
            },
        }
    }
}
