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
//! # Output
//!
//! The output of [Disassembler::disassemble] is a string that can be printed to the console or
//! written to a file. Below is an example of the disassembly for a simple "Hello, world!" program.
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────────────────┐
//! │                             <file name omitted>                              │
//! │                                                                              │
//! │                         write_line("Hello, world!")                          │
//! │                                                                              │
//! │             3 instructions, 1 constants, 0 locals, returns none              │
//! │                                                                              │
//! │                                 Instructions                                 │
//! │                                 ------------                                 │
//! │ i  BYTECODE OPERATION             INFO               TYPE         POSITION   │
//! │--- -------- ------------- -------------------- ---------------- ------------ │
//! │ 0        03 LOAD_CONSTANT R0 = C0                    str        (11, 26)     │
//! │ 1   1390117 CALL_NATIVE   write_line(R0)                        (0, 27)      │
//! │ 2        18 RETURN                                              (27, 27)     │
//! │┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈│
//! │                                    Locals                                    │
//! │                                    ------                                    │
//! │            i  IDENTIFIER       TYPE       MUTABLE  SCOPE  REGISTER           │
//! │           --- ---------- ---------------- ------- ------- --------           │
//! │┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈│
//! │                                  Constants                                   │
//! │                                  ---------                                   │
//! │                              i       VALUE                                   │
//! │                             --- ---------------                              │
//! │                              0   Hello, world!                               │
//! └──────────────────────────────────────────────────────────────────────────────┘
//! ```
use std::env::current_exe;

use colored::Colorize;

use crate::{Chunk, ConcreteValue, Local, Value};

const INSTRUCTION_HEADER: [&str; 4] = [
    "Instructions",
    "------------",
    " i  BYTECODE OPERATION             INFO               TYPE        POSITION ",
    "--- -------- ------------- -------------------- ---------------- ----------",
];

const CONSTANT_HEADER: [&str; 4] = [
    "Constants",
    "---------",
    " i       VALUE     ",
    "--- ---------------",
];

const LOCAL_HEADER: [&str; 4] = [
    "Locals",
    "------",
    " i  IDENTIFIER       TYPE       MUTABLE  SCOPE ",
    "--- ---------- ---------------- ------- -------",
];

/// Builder that constructs a human-readable representation of a chunk.
///
/// See the [module-level documentation](index.html) for more information.
pub struct Disassembler<'a> {
    output: String,
    chunk: &'a Chunk,
    source: Option<&'a str>,

    // Options
    styled: bool,
    indent: usize,
}

impl<'a> Disassembler<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        Self {
            output: String::new(),
            chunk,
            source: None,
            styled: false,
            indent: 0,
        }
    }

    /// The default width of the disassembly output. To correctly align the output, this should
    /// return the width of the longest line that the disassembler is guaranteed to produce.
    pub fn default_width() -> usize {
        let longest_line = INSTRUCTION_HEADER[3];

        longest_line.chars().count().max(80)
    }

    pub fn source(mut self, source: &'a str) -> Self {
        self.source = Some(source);

        self
    }

    pub fn styled(mut self, styled: bool) -> Self {
        self.styled = styled;

        self
    }

    pub fn indent(mut self, indent: usize) -> Self {
        self.indent = indent;

        self
    }

    fn push(
        &mut self,
        text: &str,
        center: bool,
        style_bold: bool,
        style_dim: bool,
        add_border: bool,
    ) {
        let width = Disassembler::default_width();
        let characters = text.chars().collect::<Vec<char>>();
        let content_width = if add_border { width - 2 } else { width };
        let (line_characters, remainder) = characters
            .split_at_checked(content_width)
            .unwrap_or((characters.as_slice(), &[]));
        let (left_pad_length, right_pad_length) = {
            let extra_space = content_width.saturating_sub(characters.len());

            if center {
                (extra_space / 2, extra_space / 2 + extra_space % 2)
            } else {
                (0, extra_space)
            }
        };
        let content = if style_bold {
            line_characters
                .iter()
                .collect::<String>()
                .bold()
                .to_string()
        } else if style_dim {
            line_characters
                .iter()
                .collect::<String>()
                .dimmed()
                .to_string()
        } else {
            line_characters.iter().collect::<String>()
        };
        let length_before_content = self.output.chars().count();

        for _ in 0..self.indent {
            self.output.push_str("│   ");
        }

        if add_border {
            self.output.push('│');
        }

        self.output.push_str(&" ".repeat(left_pad_length));
        self.output.push_str(&content);
        self.output.push_str(&" ".repeat(right_pad_length));

        let length_after_content = self.output.chars().count();
        let line_length = length_after_content - length_before_content;

        if line_length < content_width - 1 {
            self.output
                .push_str(&" ".repeat(content_width - line_length));
        }

        if add_border {
            self.output.push('│');
        }

        self.output.push('\n');

        if !remainder.is_empty() {
            self.push(
                remainder.iter().collect::<String>().as_str(),
                center,
                style_bold,
                style_dim,
                add_border,
            );
        }
    }

    fn push_header(&mut self, header: &str) {
        self.push(header, true, self.styled, false, true);
    }

    fn push_details(&mut self, details: &str) {
        self.push(details, true, false, false, true);
    }

    fn push_border(&mut self, border: &str) {
        self.push(border, false, false, false, false);
    }

    fn push_empty(&mut self) {
        self.push("", false, false, false, true);
    }

    pub fn disassemble(mut self) -> String {
        let width = Disassembler::default_width();
        let top_border = "┌".to_string() + &"─".repeat(width - 2) + "┐";
        let section_border = "│".to_string() + &"┈".repeat(width - 2) + "│";
        let bottom_border = "└".to_string() + &"─".repeat(width - 2) + "┘";
        let name_display = self
            .chunk
            .name()
            .map(|identifier| identifier.to_string())
            .unwrap_or_else(|| {
                current_exe()
                    .map(|path| {
                        let path_string = path.to_string_lossy();
                        let file_name = path_string
                            .split('/')
                            .last()
                            .map(|slice| slice.to_string())
                            .unwrap_or(path_string.to_string());

                        file_name
                    })
                    .unwrap_or("Chunk Disassembly".to_string())
            });

        self.push_border(&top_border);
        self.push_header(&name_display);

        if let Some(source) = self.source {
            self.push_empty();
            self.push_details(
                &source
                    .replace("  ", "")
                    .replace("\n\n", " ")
                    .replace('\n', " "),
            );
            self.push_empty();
        }

        let info_line = format!(
            "{} instructions, {} constants, {} locals, returns {}",
            self.chunk.len(),
            self.chunk.constants().len(),
            self.chunk.locals().len(),
            self.chunk.return_type()
        );

        self.push(&info_line, true, false, true, true);
        self.push_empty();

        for line in INSTRUCTION_HEADER {
            self.push_header(line);
        }

        for (index, (instruction, position)) in self.chunk.instructions().iter().enumerate() {
            let bytecode = format!("{:02X}", u32::from(instruction));
            let operation = instruction.operation().to_string();
            let info = instruction.disassembly_info(self.chunk);
            let position = position.to_string();

            let instruction_display =
                format!("{index:^3} {bytecode:>8} {operation:13} {info:^20} {position:10}");

            self.push_details(&instruction_display);
        }

        self.push_border(&section_border);

        for line in LOCAL_HEADER {
            self.push_header(line);
        }

        for (
            index,
            Local {
                identifier_index,
                r#type,
                scope,
                is_mutable: mutable,
            },
        ) in self.chunk.locals().iter().enumerate()
        {
            let identifier_display = self
                .chunk
                .constants()
                .get(*identifier_index as usize)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let type_display = r#type.to_string();
            let local_display = format!(
                "{index:^3} {identifier_display:10} {type_display:16} {mutable:7} {scope:7}"
            );

            self.push_details(&local_display);
        }

        self.push_border(&section_border);

        for line in CONSTANT_HEADER {
            self.push_header(line);
        }

        for (index, value) in self.chunk.constants().iter().enumerate() {
            let value_display = {
                let value_string = value.to_string();

                if value_string.len() > 15 {
                    format!("{value_string:.12}...")
                } else {
                    value_string
                }
            };
            let constant_display = format!("{index:^3} {value_display:^15}");

            self.push_details(&constant_display);

            if let Value::Concrete(ConcreteValue::Function(function)) = value {
                let function_disassembly = function
                    .chunk()
                    .disassembler()
                    .styled(self.styled)
                    .indent(self.indent + 1)
                    .disassemble();

                self.output.push_str(&function_disassembly);
            }
        }

        self.push_border(&bottom_border);

        let _ = self.output.trim_end_matches('\n');

        self.output
    }
}
