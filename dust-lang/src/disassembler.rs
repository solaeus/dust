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

use crate::{value::ConcreteValue, Chunk, Local};

const INSTRUCTION_HEADER: [&str; 4] = [
    "Instructions",
    "------------",
    " i   POSITION    OPERATION         TYPE                       INFO                ",
    "--- ---------- ------------- ---------------- ------------------------------------",
];

const CONSTANT_HEADER: [&str; 4] = [
    "Constants",
    "---------",
    " i        TYPE             VALUE      ",
    "--- ---------------- -----------------",
];

const LOCAL_HEADER: [&str; 4] = [
    "Locals",
    "------",
    " i  SCOPE MUTABLE       TYPE          IDENTIFIER   ",
    "--- ----- ------- ---------------- ----------------",
];

/// Builder that constructs a human-readable representation of a chunk.
///
/// See the [module-level documentation](index.html) for more information.
pub struct Disassembler<'a> {
    output: String,
    chunk: &'a Chunk,
    source: Option<&'a str>,

    // Options
    style: bool,
    indent: usize,
}

impl<'a> Disassembler<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        Self {
            output: String::new(),
            chunk,
            source: None,
            style: false,
            indent: 0,
        }
    }

    /// The default width of the disassembly output, including borders.
    pub fn default_width() -> usize {
        let longest_line = INSTRUCTION_HEADER[3];

        (longest_line.chars().count() + 2).max(80)
    }

    pub fn set_source(&mut self, source: &'a str) -> &mut Self {
        self.source = Some(source);

        self
    }

    pub fn style(mut self, styled: bool) -> Self {
        self.style = styled;

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
        let mut content = line_characters.iter().collect::<String>();

        if style_bold {
            content = content.bold().to_string();
        }

        if style_dim {
            content = content.dimmed().to_string();
        }

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

    fn push_source(&mut self, source: &str) {
        self.push(source, true, false, false, true);
    }

    fn push_chunk_info(&mut self, info: &str) {
        self.push(info, true, false, true, true);
    }

    fn push_header(&mut self, header: &str) {
        self.push(header, true, self.style, false, true);
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
            self.push_source(&source.split_whitespace().collect::<Vec<&str>>().join(" "));
            self.push_empty();
        }

        let info_line = format!(
            "{} instructions, {} constants, {} locals, returns {}",
            self.chunk.len(),
            self.chunk.constants().len(),
            self.chunk.locals().len(),
            self.chunk.r#type().return_type
        );

        self.push_chunk_info(&info_line);
        self.push_empty();

        for line in INSTRUCTION_HEADER {
            self.push_header(line);
        }

        for (index, (instruction, r#type, position)) in self.chunk.instructions().iter().enumerate()
        {
            let position = position.to_string();
            let operation = instruction.operation().to_string();
            let info = instruction.disassembly_info(self.chunk);

            let instruction_display =
                format!("{index:^3} {position:^10} {operation:13} {type:^16} {info:^36}");

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
                "{index:^3} {scope:5} {mutable:^7} {type_display:^16} {identifier_display:^16}"
            );

            self.push_details(&local_display);
        }

        self.push_border(&section_border);

        for line in CONSTANT_HEADER {
            self.push_header(line);
        }

        for (index, value) in self.chunk.constants().iter().enumerate() {
            if let ConcreteValue::Function(chunk) = value {
                let mut function_disassembler = chunk.disassembler().style(self.style);

                function_disassembler.indent = self.indent + 1;

                let function_disassembly = function_disassembler.disassemble();

                self.output.push_str(&function_disassembly);

                continue;
            }

            let type_display = value.r#type().to_string();
            let value_display = {
                let mut value_string = value.to_string();

                if value_string.len() > 15 {
                    value_string = format!("{value_string:.12}...");
                }

                value_string
            };
            let constant_display = format!("{index:^3} {type_display:^16} {value_display:^17}");

            self.push_details(&constant_display);
        }

        self.push_border(&bottom_border);

        self.output.to_string()
    }
}
