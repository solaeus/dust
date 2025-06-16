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
use std::{
    fmt::Display,
    io::{self, Write},
};

use colored::{ColoredString, Colorize};

use crate::{Address, Local, Type, chunk::Chunk};

const INSTRUCTION_COLUMNS: [(&str, usize); 4] =
    [("i", 5), ("POSITION", 12), ("OPERATION", 17), ("INFO", 41)];
const INSTRUCTION_BORDERS: [&str; 3] = [
    "╭─────┬────────────┬─────────────────┬─────────────────────────────────────────╮",
    "├─────┼────────────┼─────────────────┼─────────────────────────────────────────┤",
    "╰─────┴────────────┴─────────────────┴─────────────────────────────────────────╯",
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

const ARGUMENT_LIST_COLUMNS: [(&str, usize); 2] = [("i", 5), ("REGISTERS", 42)];
const ARGUMENT_LIST_BORDERS: [&str; 3] = [
    "╭─────┬──────────────────────────────────────────╮",
    "├─────┼──────────────────────────────────────────┤",
    "╰─────┴──────────────────────────────────────────╯",
];

const CONSTANT_COLUMNS: [(&str, usize); 3] = [("ADDRESS", 9), ("TYPE", 26), ("VALUE", 26)];
const CONSTANT_BORDERS: [&str; 3] = [
    "╭─────────┬──────────────────────────┬──────────────────────────╮",
    "├─────────┼──────────────────────────┼──────────────────────────┤",
    "╰─────────┴──────────────────────────┴──────────────────────────╯",
];

const INDENTATION: &str = "│  ";
const TOP_BORDER: [char; 3] = ['╭', '─', '╮'];
const LEFT_BORDER: char = '│';
const RIGHT_BORDER: char = '│';
const BOTTOM_BORDER: [char; 3] = ['╰', '─', '╯'];

/// Builder that constructs a human-readable representation of a chunk.
///
/// See the [module-level documentation](index.html) for more information.
pub struct Disassembler<'a, C, W> {
    chunk: &'a C,
    writer: &'a mut W,
    source: Option<&'a str>,

    // Options
    style: bool,
    indent: usize,
    width: usize,
    show_type: bool,
    show_chunk_type_name: bool,
}

impl<'a, C: Chunk, W: Write> Disassembler<'a, C, W> {
    pub fn new(chunk: &'a C, writer: &'a mut W) -> Self {
        Self {
            chunk,
            writer,
            source: None,
            style: false,
            indent: 0,
            width: Self::content_length(),
            show_type: false,
            show_chunk_type_name: true,
        }
    }

    fn indent(mut self, indent: usize) -> Self {
        self.indent = indent;

        self
    }

    pub fn source(mut self, source: &'a str) -> Self {
        self.source = Some(source);

        self
    }

    pub fn style(mut self, styled: bool) -> Self {
        self.style = styled;

        self
    }

    pub fn show_type(mut self, show_type: bool) -> Self {
        self.show_type = show_type;

        self
    }

    pub fn show_chunk_type_name(mut self, show_chunk_type_name: bool) -> Self {
        self.show_chunk_type_name = show_chunk_type_name;

        self
    }

    pub fn width(mut self, width: usize) -> Self {
        self.width = width.max(Self::content_length());

        self
    }

    fn content_length() -> usize {
        INSTRUCTION_BORDERS[0].chars().count()
    }

    fn line_length(&self) -> usize {
        let indentation_length = INDENTATION.chars().count();

        self.width + (indentation_length * self.indent) + 2 // Left and right border
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
            column_name_line.push_str(&format!("│{column_name:^width$}"));
        }

        column_name_line.push('│');
        self.write_center_border_bold("Instructions")?;
        self.write_center_border(INSTRUCTION_BORDERS[0])?;
        self.write_center_border_bold(&column_name_line)?;
        self.write_center_border(INSTRUCTION_BORDERS[1])?;

        for (index, instruction) in self.chunk.instructions().iter().enumerate() {
            let position = self.chunk.positions().map_or_else(
                || "stripped".to_string(),
                |positions| {
                    positions
                        .get(index)
                        .map(|position| position.to_string())
                        .unwrap_or_else(|| "ERROR".to_string())
                },
            );
            let operation = instruction.operation().to_string();
            let info = instruction.disassembly_info();
            let row = format!("│{index:^5}│{position:^12}│{operation:^17}│{info:^41}│");

            self.write_center_border(&row)?;
        }

        self.write_center_border(INSTRUCTION_BORDERS[2])?;

        Ok(())
    }

    fn write_local_section(&mut self) -> Result<(), io::Error> {
        let mut column_name_line = String::new();

        for (column_name, width) in LOCAL_COLUMNS {
            column_name_line.push_str(&format!("│{column_name:^width$}"));
        }

        column_name_line.push('│');
        self.write_center_border_bold("Locals")?;
        self.write_center_border(LOCAL_BORDERS[0])?;
        self.write_center_border_bold(&column_name_line)?;
        self.write_center_border(LOCAL_BORDERS[1])?;

        let locals_iterator = if let Some(iterator) = self.chunk.locals() {
            iterator
        } else {
            return Ok(());
        };

        for (
            index,
            (
                identifier,
                Local {
                    address,
                    r#type,
                    scope,
                    is_mutable,
                },
            ),
        ) in locals_iterator.enumerate()
        {
            let type_display = r#type.to_string();
            let address_display = address.to_string(r#type.kind());
            let scope = scope.to_string();
            let row = format!(
                "│{index:^5}│{identifier:^16}│{type_display:^26}│{address_display:^12}│{scope:^7}│{is_mutable:^7}│"
            );

            self.write_center_border(&row)?;
        }

        self.write_center_border(LOCAL_BORDERS[2])?;

        Ok(())
    }

    fn write_constant_section(&mut self) -> Result<(), io::Error> {
        fn write_constants<'a, C, W, T>(
            disassembler: &mut Disassembler<'a, C, W>,
            constants: &[T],
            r#type: Type,
        ) -> Result<(), io::Error>
        where
            C: Chunk,
            W: Write,
            T: Display,
        {
            for (index, value) in constants.iter().enumerate() {
                let type_display = r#type.to_string();
                let value_display = {
                    let mut value_string = value.to_string();

                    if value_string.len() > 26 {
                        value_string = format!("{value_string:.23}...");
                    }

                    value_string
                };
                let register_display = Address::constant(index as u16).to_string(r#type.kind());
                let constant_display =
                    format!("│{register_display:^9}│{type_display:^26}│{value_display:^26}│");

                disassembler.write_center_border(&constant_display)?;
            }

            Ok(())
        }

        let mut column_name_line = String::new();

        for (column_name, width) in CONSTANT_COLUMNS {
            column_name_line.push_str(&format!("│{column_name:^width$}"));
        }

        column_name_line.push('│');
        self.write_center_border_bold("Constants")?;
        self.write_center_border(CONSTANT_BORDERS[0])?;
        self.write_center_border_bold(&column_name_line)?;
        self.write_center_border(CONSTANT_BORDERS[1])?;
        write_constants(self, self.chunk.character_constants(), Type::Character)?;
        write_constants(self, self.chunk.float_constants(), Type::Float)?;
        write_constants(self, self.chunk.integer_constants(), Type::Integer)?;
        write_constants(self, self.chunk.string_constants(), Type::String)?;
        self.write_center_border(CONSTANT_BORDERS[2])?;

        Ok(())
    }

    fn write_argument_list_section(&mut self) -> Result<(), io::Error> {
        let mut column_name_line = String::new();

        for (column_name, width) in ARGUMENT_LIST_COLUMNS {
            column_name_line.push_str(&format!("│{column_name:^width$}"));
        }

        column_name_line.push('│');
        self.write_center_border_bold("Argument Lists")?;
        self.write_center_border(ARGUMENT_LIST_BORDERS[0])?;
        self.write_center_border_bold(&column_name_line)?;
        self.write_center_border(ARGUMENT_LIST_BORDERS[1])?;

        for (index, arguments) in self.chunk.arguments().iter().enumerate() {
            let argument_list_display = format!(
                "│{index:^5}│{:^42}│",
                arguments
                    .iter()
                    .map(|(address, r#type)| address.to_string(*r#type))
                    .collect::<Vec<String>>()
                    .join(", "),
            );

            self.write_center_border(&argument_list_display)?;
        }

        self.write_center_border(ARGUMENT_LIST_BORDERS[2])?;

        Ok(())
    }

    fn write_prototype_section(&mut self) -> Result<(), io::Error> {
        self.write_center_border_bold("Prototypes")?;

        for chunk in self.chunk.prototypes() {
            chunk
                .disassembler(self.writer)
                .indent(self.indent + 1)
                .width(self.width)
                .style(self.style)
                .show_type(self.show_type)
                .show_chunk_type_name(false)
                .disassemble()?;

            self.write_center_border("")?;
        }

        Ok(())
    }

    pub fn disassemble(&mut self) -> Result<(), io::Error> {
        self.write_page_border(TOP_BORDER)?;

        let name = self.chunk.name();

        if let Some(name) = &name {
            self.write_center_border_bold(name)?;
        }

        if self.show_type {
            let type_display = self.chunk.r#type().to_string();

            self.write_center_border(&type_display)?;
        }

        if let Some(source) = self.source {
            let lazily_formatted = source.split_whitespace().collect::<Vec<&str>>().join(" ");

            if name.is_some() {
                self.write_center_border("")?;
            }

            self.write_center_border(&lazily_formatted)?;
            self.write_center_border("")?;
        }

        let mut info_line = format!(
            "{} instructions, {} constants",
            self.chunk.instructions().len(),
            self.chunk.string_constants().len()
                + self.chunk.float_constants().len()
                + self.chunk.integer_constants().len()
                + self.chunk.character_constants().len(),
        );

        if let Some(locals) = self.chunk.locals() {
            let local_count = locals.count();

            info_line.push_str(&format!(", {local_count} locals"));
        }

        let return_type = &self.chunk.r#type().return_type;

        info_line.push_str(&format!(", returns {return_type}",));

        self.write_center_border_dim(&info_line)?;
        self.write_center_border("")?;

        if !self.chunk.instructions().is_empty() {
            self.write_instruction_section()?;
        }

        if self.chunk.locals().map_or(0, |locals| locals.count()) > 0 {
            self.write_local_section()?;
        }

        if !self.chunk.string_constants().is_empty()
            || !self.chunk.float_constants().is_empty()
            || !self.chunk.integer_constants().is_empty()
            || !self.chunk.string_constants().is_empty()
        {
            self.write_constant_section()?;
        }

        if !self.chunk.arguments().is_empty() {
            self.write_argument_list_section()?;
        }

        if !self.chunk.prototypes().is_empty() {
            self.write_prototype_section()?;
        }

        self.write_page_border(BOTTOM_BORDER)
    }
}
