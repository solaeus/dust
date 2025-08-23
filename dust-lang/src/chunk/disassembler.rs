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
//! ╭──────────────────────────────────────────────────────────────────────────────────╮
//! │                            write_line("hello world")                             │
//! │                                                                                  │
//! │               3 instructions, 1 constants, 0 locals, returns none                │
//! │                                                                                  │
//! │                                   Instructions                                   │
//! │ ╭─────┬────────────┬─────────────────┬─────────────────────────────────────────╮ │
//! │ │  i  │  POSITION  │    OPERATION    │                  INFO                   │ │
//! │ ├─────┼────────────┼─────────────────┼─────────────────────────────────────────┤ │
//! │ │  0  │  (11, 24)  │  LOAD_CONSTANT  │              R_STR_0 = C0               │ │
//! │ │  1  │  (0, 25)   │   CALL_NATIVE   │             write_line(R0)              │ │
//! │ │  2  │  (25, 25)  │     RETURN      │                 RETURN                  │ │
//! │ ╰─────┴────────────┴─────────────────┴─────────────────────────────────────────╯ │
//! │                                    Constants                                     │
//! │          ╭─────┬──────────────────────────┬──────────────────────────╮           │
//! │          │  i  │           TYPE           │          VALUE           │           │
//! │          ├─────┼──────────────────────────┼──────────────────────────┤           │
//! │          │  0  │           str            │       hello world        │           │
//! │          ╰─────┴──────────────────────────┴──────────────────────────╯           │
//! ╰──────────────────────────────────────────────────────────────────────────────────╯
//! ```
use std::io::{self, Write};

use colored::{ColoredString, Colorize};

use crate::{Address, Chunk};

const INSTRUCTION_COLUMNS: [(&str, usize); 3] = [("i", 5), ("OPERATION", 13), ("INFO", 41)];
const INSTRUCTION_BORDERS: [&str; 3] = [
    "╭─────┬─────────────┬─────────────────────────────────────────╮",
    "├─────┼─────────────┼─────────────────────────────────────────┤",
    "╰─────┴─────────────┴─────────────────────────────────────────╯",
];

const LOCAL_COLUMNS: [(&str, usize); 6] = [
    ("i", 5),
    ("IDENTIFIER", 16),
    ("TYPE", 26),
    ("ADDRESS", 12),
    ("SCOPE", 7),
    ("MUTABLE", 7),
];
const LOCAL_BORDERS: [&str; 3] = [
    "╭─────┬────────────────┬──────────────────────────┬────────────┬───────┬───────╮",
    "├─────┼────────────────┼──────────────────────────┼────────────┼───────┼───────┤",
    "╰─────┴────────────────┴──────────────────────────┴────────────┴───────┴───────╯",
];

const CONSTANT_COLUMNS: [(&str, usize); 3] = [("ADDRESS", 9), ("TYPE", 26), ("VALUE", 26)];
const CONSTANT_BORDERS: [&str; 3] = [
    "╭─────────┬──────────────────────────┬──────────────────────────╮",
    "├─────────┼──────────────────────────┼──────────────────────────┤",
    "╰─────────┴──────────────────────────┴──────────────────────────╯",
];

const ARGUMENT_LIST_COLUMN: (&str, usize) = ("ADDRESSES", 52);
const ARGUMENT_LIST_BORDERS: [&str; 3] = [
    "╭────────────────────────────────────────────────────╮",
    "├────────────────────────────────────────────────────┤",
    "╰────────────────────────────────────────────────────╯",
];

const DROP_LIST_COLUMNS: [(&str, usize); 2] = [("i", 5), ("REGISTERS", 46)];
const DROP_LIST_BORDERS: [&str; 3] = [
    "╭─────┬──────────────────────────────────────────────╮",
    "├─────┼──────────────────────────────────────────────┤",
    "╰─────┴──────────────────────────────────────────────╯",
];

const TOP_BORDER: [char; 3] = ['╭', '─', '╮'];
const LEFT_BORDER: char = '│';
const RIGHT_BORDER: char = '│';
const BOTTOM_BORDER: [char; 3] = ['╰', '─', '╯'];

const WIDTH: usize = 80;

/// Builder that constructs a human-readable representation of a chunk.
///
/// See the [module-level documentation](index.html) for more information.
pub struct Disassembler<'a, 'w, W> {
    chunk: &'a Chunk,
    writer: &'w mut W,
    source: Option<&'a str>,

    // Options
    style: bool,
    show_type: bool,
}

impl<'a, 'w, W: Write> Disassembler<'a, 'w, W> {
    pub fn new(chunk: &'a Chunk, writer: &'w mut W) -> Self {
        Self {
            chunk,
            writer,
            source: None,
            style: false,
            show_type: false,
        }
    }

    pub fn source(&mut self, source: &'a str) -> &mut Self {
        self.source = Some(source);

        self
    }

    pub fn style(&mut self, styled: bool) -> &mut Self {
        self.style = styled;

        self
    }

    pub fn show_type(&mut self, show_type: bool) -> &mut Self {
        self.show_type = show_type;

        self
    }

    pub fn disassemble(&mut self) -> Result<(), io::Error> {
        self.write_page_border(TOP_BORDER)?;

        let name = self.chunk.name.as_ref().map(|path| path.to_string());

        if let Some(name) = name {
            self.write_center_border_bold(name.as_ref())?;
        }

        if self.show_type {
            let type_display = self.chunk.r#type.to_string();

            self.write_center_border(&type_display)?;
        }

        if let Some(source) = self.source {
            let lazily_formatted = source.split_whitespace().collect::<Vec<&str>>().join(" ");

            if self.chunk.name.is_some() {
                self.write_center_border("")?;
            }

            self.write_center_border(&lazily_formatted)?;
            self.write_center_border("")?;
        }

        let info_line = format!(
            "{} instructions, {} constants, returns {}",
            self.chunk.instructions.len(),
            self.chunk.constants.len(),
            self.chunk.r#type.return_type
        );

        self.write_center_border_dim(&info_line)?;
        self.write_center_border("")?;

        if !self.chunk.instructions.is_empty() {
            self.write_instruction_section(self.chunk)?;
        }

        if !self.chunk.constants.is_empty() {
            self.write_constant_section(self.chunk)?;
        }

        if !self.chunk.call_arguments.is_empty() {
            self.write_call_arguments_section(self.chunk)?;
        }

        // if !chunk.drop_lists.is_empty() {
        //     self.write_drop_list_section(chunk)?;
        // }

        self.write_page_border(BOTTOM_BORDER)
    }

    fn line_length(&self) -> usize {
        WIDTH + 2 // Left and right border
    }

    fn write_char(&mut self, character: char) -> Result<(), io::Error> {
        write!(&mut self.writer, "{character}")
    }

    fn write_colored(&mut self, text: &ColoredString) -> Result<(), io::Error> {
        write!(&mut self.writer, "{text}")
    }

    fn write_str(&mut self, text: &str) -> Result<(), io::Error> {
        write!(&mut self.writer, "{text}")
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
            let character_count = text.chars().count();

            if character_count > WIDTH {
                let split_index = text
                    .char_indices()
                    .nth(WIDTH)
                    .map(|(index, _)| index)
                    .unwrap_or_else(|| character_count);

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
        self.write_char(border[0])?;

        for _ in 0..self.line_length() {
            self.write_char(border[1])?;
        }

        self.write_char(border[2])?;
        self.write_char('\n')
    }

    fn write_instruction_section(&mut self, chunk: &Chunk) -> Result<(), io::Error> {
        let mut column_name_line = String::new();

        for (column_name, width) in INSTRUCTION_COLUMNS {
            column_name_line.push_str(&format!("│{column_name:^width$}"));
        }

        column_name_line.push('│');
        self.write_center_border_bold("Instructions")?;
        self.write_center_border(INSTRUCTION_BORDERS[0])?;
        self.write_center_border_bold(&column_name_line)?;
        self.write_center_border(INSTRUCTION_BORDERS[1])?;

        for (index, instruction) in chunk.instructions.iter().enumerate() {
            let operation = instruction.operation().to_string();
            let info = instruction.disassembly_info();
            let row = format!("│{index:^5}│{operation:^13}│{info:^41}│");

            self.write_center_border(&row)?;
        }

        self.write_center_border(INSTRUCTION_BORDERS[2])?;

        Ok(())
    }

    fn write_constant_section(&mut self, chunk: &Chunk) -> Result<(), io::Error> {
        let mut column_name_line = String::new();

        for (column_name, width) in CONSTANT_COLUMNS {
            column_name_line.push_str(&format!("│{column_name:^width$}"));
        }

        column_name_line.push('│');
        self.write_center_border_bold("Constants")?;
        self.write_center_border(CONSTANT_BORDERS[0])?;
        self.write_center_border_bold(&column_name_line)?;
        self.write_center_border(CONSTANT_BORDERS[1])?;

        for (index, value) in chunk.constants.iter().enumerate() {
            let r#type = value.operand_type();
            let type_display = r#type.to_string();
            let value_display = {
                let mut value_string = value.to_string();

                if value_string.chars().count() > 26 {
                    value_string = format!("{value_string:.23}...");
                }

                value_string
            };
            let register_display = Address::constant(index as u16).to_string(r#type);
            let constant_display =
                format!("│{register_display:^9}│{type_display:^26}│{value_display:^26}│");

            self.write_center_border(&constant_display)?;
        }

        self.write_center_border(CONSTANT_BORDERS[2])?;

        Ok(())
    }

    fn write_call_arguments_section(&mut self, chunk: &Chunk) -> Result<(), io::Error> {
        let mut column_name_line = String::new();
        let (column_name, width) = ARGUMENT_LIST_COLUMN;

        column_name_line.push_str(&format!("│{column_name:^width$}│"));
        self.write_center_border_bold("Argument Lists")?;
        self.write_center_border(ARGUMENT_LIST_BORDERS[0])?;
        self.write_center_border_bold(&column_name_line)?;
        self.write_center_border(ARGUMENT_LIST_BORDERS[1])?;

        let row = format!("│{:52?}│", chunk.call_arguments);

        self.write_center_border(&row)?;
        self.write_center_border(ARGUMENT_LIST_BORDERS[2])?;

        Ok(())
    }

    // fn write_drop_list_section(&mut self, chunk: &Chunk) -> Result<(), io::Error> {
    //     let mut column_name_line = String::new();

    //     for (column_name, width) in DROP_LIST_COLUMNS {
    //         column_name_line.push_str(&format!("│{column_name:^width$}"));
    //     }

    //     column_name_line.push('│');
    //     self.write_center_border_bold("Drop Lists")?;
    //     self.write_center_border(DROP_LIST_BORDERS[0])?;
    //     self.write_center_border_bold(&column_name_line)?;
    //     self.write_center_border(DROP_LIST_BORDERS[1])?;

    //     for (index, drop_list) in chunk.drop_lists.iter().enumerate() {
    //         let registers_display = drop_list
    //             .iter()
    //             .map(|index| format!("reg_{index}"))
    //             .collect::<Vec<_>>()
    //             .join(", ");
    //         let row = format!("│{index:^5}│{registers_display:^46}│");

    //         self.write_center_border(&row)?;
    //     }

    //     self.write_center_border(DROP_LIST_BORDERS[2])?;

    //     Ok(())
    // }
}
