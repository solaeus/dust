//! Tool for disassembling chunks into a human-readable format.
//!
//! A disassembler can be created by calling [Chunk::disassembler][] or by instantiating one with
//! [Disassembler::new][].
//!
//! # Options
//!
//! The disassembler can be customized with the 'styled' option, which will apply ANSI color codes
//! to the output.
//!
//! If the 'source' option is set, the disassembler will include the source code in the output.
//!
//! # Output
//!
//! The disassembler will output a human-readable representation of the chunk by writing to any type
//! that implements the [Write][] trait.
//!
//! ```text
//! ╭─────────────────────────────────────────────────────────────────╮
//! │                              dust                               │
//! │                                                                 │
//! │                   write_line("Hello world!")                    │
//! │                                                                 │
//! │      3 instructions, 1 constants, 0 locals, returns none        │
//! │                                                                 │
//! │                          Instructions                           │
//! │ ╭─────┬────────────┬─────────────────┬────────────────────────╮ │
//! │ │  i  │  POSITION  │    OPERATION    │          INFO          │ │
//! │ ├─────┼────────────┼─────────────────┼────────────────────────┤ │
//! │ │  0  │  (11, 25)  │  LOAD_CONSTANT  │        R0 = C0         │ │
//! │ │  1  │  (0, 26)   │   CALL_NATIVE   │     write_line(R0)     │ │
//! │ │  2  │  (26, 26)  │     RETURN      │         RETURN         │ │
//! │ ╰─────┴────────────┴─────────────────┴────────────────────────╯ │
//! │                            Constants                            │
//! │  ╭─────┬──────────────────────────┬──────────────────────────╮  │
//! │  │  i  │           TYPE           │          VALUE           │  │
//! │  ├─────┼──────────────────────────┼──────────────────────────┤  │
//! │  │  0  │           str            │       Hello world!       │  │
//! │  ╰─────┴──────────────────────────┴──────────────────────────╯  │
//! ╰─────────────────────────────────────────────────────────────────╯
//! ```
use std::io::{self, Write};

use colored::{ColoredString, Colorize};

use crate::{Chunk, Local};

const INSTRUCTION_COLUMNS: [(&str, usize); 4] =
    [("i", 5), ("POSITION", 12), ("OPERATION", 17), ("INFO", 24)];
const INSTRUCTION_BORDERS: [&str; 3] = [
    "╭─────┬────────────┬─────────────────┬────────────────────────╮",
    "├─────┼────────────┼─────────────────┼────────────────────────┤",
    "╰─────┴────────────┴─────────────────┴────────────────────────╯",
];

const LOCAL_COLUMNS: [(&str, usize); 5] = [
    ("i", 5),
    ("IDENTIFIER", 16),
    ("REGISTER", 10),
    ("SCOPE", 7),
    ("MUTABLE", 7),
];
const LOCAL_BORDERS: [&str; 3] = [
    "╭─────┬────────────────┬──────────┬───────┬───────╮",
    "├─────┼────────────────┼──────────┼───────┼───────┤",
    "╰─────┴────────────────┴──────────┴───────┴───────╯",
];

const CONSTANT_COLUMNS: [(&str, usize); 3] = [("i", 5), ("TYPE", 26), ("VALUE", 26)];
const CONSTANT_BORDERS: [&str; 3] = [
    "╭─────┬──────────────────────────┬──────────────────────────╮",
    "├─────┼──────────────────────────┼──────────────────────────┤",
    "╰─────┴──────────────────────────┴──────────────────────────╯",
];

const INDENTATION: &str = "│  ";
const TOP_BORDER: [char; 3] = ['╭', '─', '╮'];
const LEFT_BORDER: char = '│';
const RIGHT_BORDER: char = '│';
const BOTTOM_BORDER: [char; 3] = ['╰', '─', '╯'];

/// Builder that constructs a human-readable representation of a chunk.
///
/// See the [module-level documentation](index.html) for more information.
pub struct Disassembler<'a, W> {
    writer: &'a mut W,
    chunk: &'a Chunk,
    source: Option<&'a str>,

    // Options
    style: bool,
    indent: usize,
    width: usize,
    show_type: bool,
}

impl<'a, W: Write> Disassembler<'a, W> {
    pub fn new(chunk: &'a Chunk, writer: &'a mut W) -> Self {
        Self {
            writer,
            chunk,
            source: None,
            style: false,
            indent: 0,
            width: Self::content_length(),
            show_type: false,
        }
    }

    pub fn source(mut self, source: &'a str) -> Self {
        self.source = Some(source);

        self
    }

    pub fn style(mut self, styled: bool) -> Self {
        self.style = styled;

        self
    }

    pub fn width(mut self, width: usize) -> Self {
        self.width = width.max(Self::content_length());

        self
    }

    pub fn show_type(mut self, show_type: bool) -> Self {
        self.show_type = show_type;

        self
    }

    fn indent(mut self, indent: usize) -> Self {
        self.indent = indent;

        self
    }

    fn content_length() -> usize {
        let longest_line_length = INSTRUCTION_BORDERS[0].chars().count();

        longest_line_length
    }

    fn line_length(&self) -> usize {
        let indentation_length = INDENTATION.chars().count();

        self.width + (indentation_length * self.indent) + 2 // Left and right border
    }

    fn write_char(&mut self, c: char) -> Result<(), io::Error> {
        write!(&mut self.writer, "{}", c)
    }

    fn write_colored(&mut self, text: &ColoredString) -> Result<(), io::Error> {
        write!(&mut self.writer, "{}", text)
    }

    fn write_str(&mut self, text: &str) -> Result<(), io::Error> {
        write!(&mut self.writer, "{}", text)
    }

    fn write_content(
        &mut self,
        text: &str,
        center: bool,
        style_bold: bool,
        style_dim: bool,
        add_border: bool,
    ) -> Result<(), io::Error> {
        let (line_content, overflow) = {
            if text.len() > self.width {
                let split_index = text
                    .char_indices()
                    .nth(self.width)
                    .map(|(index, _)| index)
                    .unwrap_or_else(|| text.len());

                text.split_at(split_index)
            } else {
                (text, "")
            }
        };
        let (left_pad_length, right_pad_length) = {
            let width = self.line_length();
            let line_content_length = line_content.chars().count();
            let extra_space = width.saturating_sub(line_content_length);
            let half = extra_space / 2;
            let remainder = extra_space % 2;

            if center {
                (half, half + remainder)
            } else {
                (0, extra_space)
            }
        };

        for _ in 0..self.indent {
            self.write_str(INDENTATION)?;
        }

        if add_border {
            self.write_char(LEFT_BORDER)?;
        }

        if center {
            for _ in 0..left_pad_length {
                self.write_char(' ')?;
            }
        }

        if style_bold {
            self.write_colored(&line_content.bold())?;
        } else if style_dim {
            self.write_colored(&line_content.dimmed())?;
        } else {
            self.write_str(line_content)?;
        }

        if center {
            for _ in 0..right_pad_length {
                self.write_char(' ')?;
            }
        }

        if add_border {
            self.write_char(RIGHT_BORDER)?;
        }

        self.write_char('\n')?;

        if !overflow.is_empty() {
            self.write_content(overflow, center, style_bold, style_dim, add_border)?;
        }

        Ok(())
    }

    fn write_center_border(&mut self, text: &str) -> Result<(), io::Error> {
        self.write_content(text, true, false, false, true)
    }

    fn write_center_border_dim(&mut self, text: &str) -> Result<(), io::Error> {
        self.write_content(text, true, false, self.style, true)
    }

    fn write_center_border_bold(&mut self, text: &str) -> Result<(), io::Error> {
        self.write_content(text, true, self.style, false, true)
    }

    fn write_page_border(&mut self, border: [char; 3]) -> Result<(), io::Error> {
        for _ in 0..self.indent {
            self.write_str(INDENTATION)?;
        }

        self.write_char(border[0])?;

        for _ in 0..self.line_length() {
            self.write_char(border[1])?;
        }

        self.write_char(border[2])?;
        self.write_char('\n')
    }

    fn write_instruction_section(&mut self) -> Result<(), io::Error> {
        let mut column_name_line = String::new();

        for (column_name, width) in INSTRUCTION_COLUMNS {
            column_name_line.push_str(&format!("│{column_name:^width$}", width = width));
        }

        column_name_line.push('│');
        self.write_center_border_bold("Instructions")?;
        self.write_center_border(INSTRUCTION_BORDERS[0])?;
        self.write_center_border(&column_name_line)?;
        self.write_center_border(INSTRUCTION_BORDERS[1])?;

        for (index, instruction) in self.chunk.instructions().iter().enumerate() {
            let position = self
                .chunk
                .positions()
                .get(index)
                .map(|position| position.to_string())
                .unwrap_or("stripped".to_string());
            let operation = instruction.operation().to_string();
            let info = instruction.disassembly_info();
            let row = format!("│{index:^5}│{position:^12}│{operation:^17}│{info:^24}│");

            self.write_center_border(&row)?;
        }

        self.write_center_border(INSTRUCTION_BORDERS[2])?;

        Ok(())
    }

    fn write_local_section(&mut self) -> Result<(), io::Error> {
        let mut column_name_line = String::new();

        for (column_name, width) in LOCAL_COLUMNS {
            column_name_line.push_str(&format!("│{:^width$}", column_name, width = width));
        }

        column_name_line.push('│');
        self.write_center_border_bold("Locals")?;
        self.write_center_border(LOCAL_BORDERS[0])?;
        self.write_center_border(&column_name_line)?;
        self.write_center_border(LOCAL_BORDERS[1])?;

        for (
            index,
            Local {
                identifier_index,
                register_index,
                scope,
                is_mutable,
            },
        ) in self.chunk.locals().iter().enumerate()
        {
            let identifier_display = self
                .chunk
                .constants()
                .get(*identifier_index as usize)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let register_display = format!("R{register_index}");
            let scope = scope.to_string();
            let row = format!(
                "│{index:^5}│{identifier_display:^16}│{register_display:^10}│{scope:^7}│{is_mutable:^7}│"
            );

            self.write_center_border(&row)?;
        }

        self.write_center_border(LOCAL_BORDERS[2])?;

        Ok(())
    }

    fn write_constant_section(&mut self) -> Result<(), io::Error> {
        let mut column_name_line = String::new();

        for (column_name, width) in CONSTANT_COLUMNS {
            column_name_line.push_str(&format!("│{:^width$}", column_name, width = width));
        }

        column_name_line.push('│');
        self.write_center_border_bold("Constants")?;
        self.write_center_border(CONSTANT_BORDERS[0])?;
        self.write_center_border(&column_name_line)?;
        self.write_center_border(CONSTANT_BORDERS[1])?;

        for (index, value) in self.chunk.constants().iter().enumerate() {
            let type_display = value.r#type().to_string();
            let value_display = {
                let mut value_string = value.to_string();

                if value_string.len() > 26 {
                    value_string = format!("{value_string:.23}...");
                }

                value_string
            };
            let constant_display = format!("│{index:^5}│{type_display:^26}│{value_display:^26}│");

            self.write_center_border(&constant_display)?;
        }

        self.write_center_border(CONSTANT_BORDERS[2])?;

        Ok(())
    }

    pub fn write_prototype_section(&mut self) -> Result<(), io::Error> {
        self.write_center_border_bold("Functions")?;

        for chunk in &self.chunk.prototypes {
            chunk
                .disassembler(self.writer)
                .indent(self.indent + 1)
                .width(self.width)
                .style(true)
                .show_type(true)
                .disassemble()?;

            self.write_center_border("")?;
        }

        Ok(())
    }

    pub fn disassemble(&mut self) -> Result<(), io::Error> {
        self.write_page_border(TOP_BORDER)?;

        if let Some(name) = &self.chunk.name {
            self.write_center_border_bold(name)?;
        }

        if self.show_type {
            let type_display = self.chunk.r#type.to_string();

            self.write_center_border(&type_display)?;
        }

        if let Some(source) = self.source {
            let lazily_formatted = source.split_whitespace().collect::<Vec<&str>>().join(" ");

            self.write_center_border("")?;
            self.write_center_border(&lazily_formatted)?;
            self.write_center_border("")?;
        }

        let info_line = format!(
            "{} instructions, {} constants, {} locals, returns {}",
            self.chunk.instructions.len(),
            self.chunk.constants.len(),
            self.chunk.locals.len(),
            self.chunk.r#type.return_type
        );

        self.write_center_border_dim(&info_line)?;
        self.write_center_border("")?;

        if !self.chunk.instructions.is_empty() {
            self.write_instruction_section()?;
        }

        if !self.chunk.locals.is_empty() {
            self.write_local_section()?;
        }

        if !self.chunk.constants.is_empty() {
            self.write_constant_section()?;
        }

        if !self.chunk.prototypes.is_empty() {
            self.write_prototype_section()?;
        }

        self.write_page_border(BOTTOM_BORDER)
    }
}
