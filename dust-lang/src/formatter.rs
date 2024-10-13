use std::mem::replace;

use colored::{ColoredString, Colorize, CustomColor};

use crate::{DustError, LexError, Lexer, Token};

pub fn format(source: &str, line_numbers: bool, colored: bool) -> Result<String, DustError> {
    let lexer = Lexer::new(source);
    let formatted = Formatter::new(lexer)
        .line_numbers(line_numbers)
        .colored(colored)
        .format()
        .map_err(|error| DustError::Lex { error, source })?;

    Ok(formatted)
}

#[derive(Debug)]
pub struct Formatter<'src> {
    lexer: Lexer<'src>,
    output_lines: Vec<(String, LineKind, usize)>,
    next_line: String,
    indent: usize,

    current_token: Token<'src>,
    previous_token: Token<'src>,

    // Options
    line_numbers: bool,
    colored: bool,
}

impl<'src> Formatter<'src> {
    pub fn new(mut lexer: Lexer<'src>) -> Self {
        let (current_token, _) = lexer.next_token().unwrap();

        Self {
            lexer,
            output_lines: Vec::new(),
            next_line: String::new(),
            indent: 0,
            current_token,
            previous_token: Token::Eof,
            line_numbers: false,
            colored: false,
        }
    }

    pub fn line_numbers(mut self, line_numbers: bool) -> Self {
        self.line_numbers = line_numbers;

        self
    }

    pub fn colored(mut self, colored: bool) -> Self {
        self.colored = colored;

        self
    }

    pub fn format(&mut self) -> Result<String, LexError> {
        let mut line_kind = LineKind::Empty;

        self.advance()?;

        while self.current_token != Token::Eof {
            use Token::*;

            if self.current_token.is_expression() && line_kind != LineKind::Assignment {
                line_kind = LineKind::Expression;
            }

            match self.current_token {
                Boolean(boolean) => {
                    self.push_colored(boolean.red());
                }
                Byte(byte) => {
                    self.push_colored(byte.green());
                }
                Character(character) => {
                    self.push_colored(
                        character
                            .to_string()
                            .custom_color(CustomColor::new(225, 150, 150)),
                    );
                }
                Float(float) => {
                    self.push_colored(float.yellow());
                }
                Identifier(identifier) => {
                    self.push_colored(identifier.blue());
                    self.next_line.push(' ');
                }
                Integer(integer) => {
                    self.push_colored(integer.cyan());
                }
                String(string) => {
                    self.push_colored(string.magenta());
                }
                LeftCurlyBrace => {
                    self.next_line.push_str(self.current_token.as_str());
                    self.commit_line(LineKind::OpenBlock);

                    self.indent += 1;
                }
                RightCurlyBrace => {
                    self.commit_line(LineKind::CloseBlock);
                    self.next_line.push_str(self.current_token.as_str());

                    self.indent -= 1;
                }
                Semicolon => {
                    if line_kind != LineKind::Assignment {
                        line_kind = LineKind::Statement;
                    }

                    self.next_line.push_str(self.current_token.as_str());
                    self.commit_line(line_kind);
                }
                Let => {
                    line_kind = LineKind::Assignment;

                    self.push_colored(self.current_token.as_str().bold());
                    self.next_line.push(' ');
                }
                Break | Loop | Return | While => {
                    line_kind = LineKind::Statement;

                    self.push_colored(self.current_token.as_str().bold());
                    self.next_line.push(' ');
                }
                token => {
                    self.next_line.push_str(token.as_str());
                    self.next_line.push(' ');
                }
            }
        }

        let mut previous_index = 0;
        let mut current_index = 1;

        while current_index < self.output_lines.len() {
            let (_, previous, _) = &self.output_lines[previous_index];
            let (_, current, _) = &self.output_lines[current_index];

            match (previous, current) {
                (LineKind::Empty, _)
                | (_, LineKind::Empty)
                | (LineKind::OpenBlock, _)
                | (_, LineKind::CloseBlock) => {}
                (left, right) if left == right => {}
                _ => {
                    self.output_lines
                        .insert(current_index, ("".to_string(), LineKind::Empty, 0));
                }
            }

            previous_index += 1;
            current_index += 1;
        }

        let formatted = String::with_capacity(
            self.output_lines
                .iter()
                .fold(0, |total, (line, _, _)| total + line.len()),
        );

        Ok(self.output_lines.iter().enumerate().fold(
            formatted,
            |acc, (index, (line, _, indent))| {
                let index = if index == 0 {
                    format!("{:<3}| ", index + 1).dimmed()
                } else {
                    format!("\n{:<3}| ", index + 1).dimmed()
                };
                let left_pad = "    ".repeat(*indent);

                format!("{}{}{}{}", acc, index, left_pad, line)
            },
        ))
    }

    fn advance(&mut self) -> Result<(), LexError> {
        if self.lexer.is_eof() {
            return Ok(());
        }

        let (new_token, position) = self.lexer.next_token()?;

        log::info!(
            "Parsing {} at {}",
            new_token.to_string().bold(),
            position.to_string()
        );

        self.previous_token = replace(&mut self.current_token, new_token);

        Ok(())
    }

    fn push_colored(&mut self, colored: ColoredString) {
        self.next_line.push_str(&format!("{}", colored));
    }

    fn commit_line(&mut self, line_kind: LineKind) {
        self.output_lines
            .push((self.next_line.clone(), line_kind, self.indent));
        self.next_line.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineKind {
    Empty,
    Assignment,
    Expression,
    Statement,
    OpenBlock,
    CloseBlock,
    Call,
    Primary,
}
