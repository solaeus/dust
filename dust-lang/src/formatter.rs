use colored::{ColoredString, Colorize, CustomColor};

use crate::{lex, Token};

#[derive(Debug)]
pub struct Formatter<'src> {
    source: &'src str,
    lines: Vec<(String, LineKind, usize)>,
    next_line: String,
    indent: usize,
}

impl<'src> Formatter<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            lines: Vec::new(),
            next_line: String::new(),
            indent: 0,
        }
    }

    pub fn footer(&mut self, footer: &'src str) -> &mut Self {
        self.source = footer;

        self
    }

    pub fn format(&mut self) -> String {
        let tokens = match lex(self.source) {
            Ok(tokens) => tokens,
            Err(error) => return format!("{}", error),
        };
        let mut line_kind = LineKind::Empty;

        for (token, _) in tokens {
            use Token::*;

            match token {
                Boolean(boolean) => {
                    self.push_colored(boolean.red());

                    if line_kind != LineKind::Assignment {
                        line_kind = LineKind::Expression;
                    }
                }
                Byte(byte) => {
                    self.push_colored(byte.green());

                    if line_kind != LineKind::Assignment {
                        line_kind = LineKind::Expression;
                    }
                }
                Character(character) => {
                    self.push_colored(
                        character
                            .to_string()
                            .custom_color(CustomColor::new(225, 150, 150)),
                    );

                    if line_kind != LineKind::Assignment {
                        line_kind = LineKind::Expression;
                    }
                }
                Float(float) => {
                    self.push_colored(float.yellow());

                    if line_kind != LineKind::Assignment {
                        line_kind = LineKind::Expression;
                    }
                }
                Identifier(identifier) => {
                    self.push_colored(identifier.blue());
                    self.next_line.push(' ');

                    if line_kind != LineKind::Assignment {
                        line_kind = LineKind::Expression;
                    }
                }
                Integer(integer) => {
                    self.push_colored(integer.cyan());

                    if line_kind != LineKind::Assignment {
                        line_kind = LineKind::Expression;
                    }
                }
                String(string) => {
                    self.push_colored(string.magenta());

                    if line_kind != LineKind::Assignment {
                        line_kind = LineKind::Expression;
                    }
                }
                LeftCurlyBrace => {
                    self.next_line.push_str(token.as_str());
                    self.commit_line(LineKind::OpenBlock);

                    self.indent += 1;
                }
                RightCurlyBrace => {
                    self.commit_line(LineKind::CloseBlock);
                    self.next_line.push_str(token.as_str());

                    self.indent -= 1;
                }
                Semicolon => {
                    if line_kind != LineKind::Assignment {
                        line_kind = LineKind::Statement;
                    }

                    self.next_line.push_str(token.as_str());
                    self.commit_line(line_kind);
                }
                Let => {
                    line_kind = LineKind::Assignment;

                    self.push_colored(token.as_str().bold());
                    self.next_line.push(' ');
                }
                Break | Loop | Return | While => {
                    line_kind = LineKind::Statement;

                    self.push_colored(token.as_str().bold());
                    self.next_line.push(' ');
                }
                Eof => continue,
                token => {
                    self.next_line.push_str(token.as_str());
                    self.next_line.push(' ');
                }
            }
        }

        let mut previous_index = 0;
        let mut current_index = 1;

        while current_index < self.lines.len() {
            let (_, previous, _) = &self.lines[previous_index];
            let (_, current, _) = &self.lines[current_index];

            println!("{:?} {:?}", previous, current);

            match (previous, current) {
                (LineKind::Empty, _)
                | (_, LineKind::Empty)
                | (LineKind::OpenBlock, _)
                | (_, LineKind::CloseBlock) => {}
                (left, right) if left == right => {}
                _ => {
                    self.lines
                        .insert(current_index, ("".to_string(), LineKind::Empty, 0));
                }
            }

            previous_index += 1;
            current_index += 1;
        }

        let formatted = String::with_capacity(
            self.lines
                .iter()
                .fold(0, |total, (line, _, _)| total + line.len()),
        );

        self.lines
            .iter()
            .enumerate()
            .fold(formatted, |acc, (index, (line, _, indent))| {
                let left_pad = "    ".repeat(*indent);

                if index == 0 {
                    return format!("{:<3}| {}{}", index + 1, left_pad, line);
                }
                format!("{}\n{:<3}| {}{}", acc, index + 1, left_pad, line)
            })
    }

    fn push_colored(&mut self, colored: ColoredString) {
        self.next_line.push_str(&format!("{}", colored));
    }

    fn commit_line(&mut self, line_kind: LineKind) {
        self.lines
            .push((self.next_line.clone(), line_kind, self.indent));
        self.next_line.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineKind {
    Assignment,
    FunctionCall,
    Statement,
    Expression,
    Empty,
    OpenBlock,
    CloseBlock,
}
