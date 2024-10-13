use annotate_snippets::{renderer::Style, Level, Renderer, Snippet};
use colored::{Colorize, CustomColor};

use crate::{lex, Token};

#[derive(Debug, Copy, Clone)]
pub struct Formatter<'src> {
    source: &'src str,
    origin: Option<&'src str>,
    footer: Option<&'src str>,
}

impl<'src> Formatter<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            origin: None,
            footer: None,
        }
    }

    pub fn origin(&mut self, origin: &'src str) -> &mut Self {
        self.origin = Some(origin);

        self
    }

    pub fn footer(&mut self, footer: &'src str) -> &mut Self {
        self.source = footer;

        self
    }

    pub fn format(&self) -> String {
        let tokens = match lex(self.source) {
            Ok(tokens) => tokens,
            Err(error) => return format!("{}", error),
        };
        let mut block_depth = 0;
        let mut formatted = String::new();
        let line_break = |formatted: &mut String, block_depth: i32| {
            formatted.push('\n');

            for _ in 0..block_depth {
                formatted.push_str("    ");
            }
        };

        for (token, _) in tokens {
            match token {
                Token::Boolean(boolean) => formatted.push_str(&boolean.red()),
                Token::Byte(byte) => formatted.push_str(&byte.green()),
                Token::Character(character) => formatted.push_str(
                    &character
                        .to_string()
                        .custom_color(CustomColor::new(225, 150, 150)),
                ),
                Token::Float(float) => formatted.push_str(&float.yellow()),
                Token::Identifier(identifier) => {
                    formatted.push_str(&identifier.blue());
                    formatted.push(' ');
                }
                Token::Integer(integer) => formatted.push_str(&integer.cyan()),
                Token::String(string) => formatted.push_str(&string.magenta()),
                Token::LeftCurlyBrace => {
                    block_depth += 1;

                    formatted.push_str(token.as_str());
                    line_break(&mut formatted, block_depth)
                }
                Token::RightCurlyBrace => {
                    block_depth -= 1;

                    line_break(&mut formatted, block_depth);
                    formatted.push_str(token.as_str());
                }
                Token::Semicolon => {
                    formatted.push_str(token.as_str());
                    line_break(&mut formatted, block_depth);
                }
                Token::Eof => continue,
                token => {
                    formatted.push_str(token.as_str());
                    formatted.push(' ');
                }
            }
        }

        formatted
    }
}
