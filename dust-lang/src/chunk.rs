use std::{
    env::current_exe,
    fmt::{self, Debug, Display},
    rc::Weak,
};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{Instruction, Span, Type, Value};

#[derive(Clone, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Chunk {
    name: Option<String>,
    instructions: Vec<(Instruction, Span)>,
    constants: Vec<Value>,
    locals: Vec<Local>,
    current_scope: Scope,
    block_count: usize,
}

impl Chunk {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            instructions: Vec::new(),
            constants: Vec::new(),
            locals: Vec::new(),
            current_scope: Scope::default(),
            block_count: 0,
        }
    }

    pub fn with_data(
        name: Option<String>,
        instructions: Vec<(Instruction, Span)>,
        constants: Vec<Value>,
        locals: Vec<Local>,
    ) -> Self {
        Self {
            name,
            instructions,
            constants,
            locals,
            current_scope: Scope::default(),
            block_count: 0,
        }
    }

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    pub fn constants(&self) -> &Vec<Value> {
        &self.constants
    }

    pub fn constants_mut(&mut self) -> &mut Vec<Value> {
        &mut self.constants
    }

    pub fn take_constants(self) -> Vec<Value> {
        self.constants
    }

    pub fn instructions(&self) -> &Vec<(Instruction, Span)> {
        &self.instructions
    }

    pub fn instructions_mut(&mut self) -> &mut Vec<(Instruction, Span)> {
        &mut self.instructions
    }

    pub fn locals(&self) -> &Vec<Local> {
        &self.locals
    }

    pub fn locals_mut(&mut self) -> &mut Vec<Local> {
        &mut self.locals
    }

    pub fn current_scope(&self) -> Scope {
        self.current_scope
    }

    pub fn get_constant(&self, index: u8) -> Option<&Value> {
        self.constants.get(index as usize)
    }

    pub fn push_or_get_constant(&mut self, value: Value) -> u8 {
        if let Some(index) = self
            .constants
            .iter()
            .position(|constant| constant == &value)
        {
            return index as u8;
        }

        self.constants.push(value);

        (self.constants.len() - 1) as u8
    }

    pub fn get_identifier(&self, local_index: u8) -> Option<String> {
        self.locals.get(local_index as usize).and_then(|local| {
            self.constants
                .get(local.identifier_index as usize)
                .map(|value| value.to_string())
        })
    }

    pub fn begin_scope(&mut self) {
        self.current_scope.depth += 1;
        self.current_scope.block = self.block_count;
    }

    pub fn end_scope(&mut self) {
        self.current_scope.depth -= 1;

        if self.current_scope.depth == 0 {
            self.block_count += 1;
            self.current_scope.block = 0;
        } else {
            self.current_scope.block = self.block_count;
        };
    }

    pub fn disassembler(&self) -> ChunkDisassembler {
        ChunkDisassembler::new(self)
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.disassembler().styled(true).disassemble())
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disassembly = self.disassembler().styled(false).disassemble();

        if cfg!(debug_assertions) {
            write!(f, "\n{}", disassembly)
        } else {
            write!(f, "{}", disassembly)
        }
    }
}

impl Eq for Chunk {}

impl PartialEq for Chunk {
    fn eq(&self, other: &Self) -> bool {
        self.instructions == other.instructions
            && self.constants == other.constants
            && self.locals == other.locals
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Local {
    pub identifier_index: u8,
    pub r#type: Option<Type>,
    pub is_mutable: bool,
    pub scope: Scope,
    pub register_index: u8,
}

impl Local {
    pub fn new(
        identifier_index: u8,
        r#type: Option<Type>,
        mutable: bool,
        scope: Scope,
        register_index: u8,
    ) -> Self {
        Self {
            identifier_index,
            r#type,
            is_mutable: mutable,
            scope,
            register_index,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Scope {
    /// The level of block nesting.
    pub depth: usize,
    /// The nth top-level block in the chunk.
    pub block: usize,
}

impl Scope {
    pub fn new(depth: usize, block: usize) -> Self {
        Self { depth, block }
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.depth, self.block)
    }
}

pub struct ChunkDisassembler<'a> {
    chunk: &'a Chunk,
    source: Option<&'a str>,
    width: usize,
    styled: bool,
    indent: usize,
}

impl<'a> ChunkDisassembler<'a> {
    const INSTRUCTION_HEADER: [&'static str; 4] = [
        "Instructions",
        "------------",
        "INDEX BYTECODE OPERATION       INFO                      POSITION     ",
        "----- -------- --------------- ------------------------- -------------",
    ];

    const CONSTANT_HEADER: [&'static str; 4] =
        ["Constants", "---------", "INDEX VALUE", "----- -----"];

    const LOCAL_HEADER: [&'static str; 4] = [
        "Locals",
        "------",
        "INDEX IDENTIFIER TYPE     MUTABLE SCOPE REGISTER",
        "----- ---------- -------- ------- ----- --------",
    ];

    /// The default width of the disassembly output. To correctly align the output, this should
    /// return the width of the longest line that the disassembler is guaranteed to produce.
    pub fn default_width() -> usize {
        let longest_line = Self::INSTRUCTION_HEADER[3];

        longest_line.chars().count().max(80)
    }

    pub fn new(chunk: &'a Chunk) -> Self {
        Self {
            chunk,
            source: None,
            width: Self::default_width(),
            styled: false,
            indent: 0,
        }
    }

    pub fn source(&mut self, source: &'a str) -> &mut Self {
        self.source = Some(source);

        self
    }

    pub fn width(&mut self, width: usize) -> &mut Self {
        self.width = width;

        self
    }

    pub fn styled(&mut self, styled: bool) -> &mut Self {
        self.styled = styled;

        self
    }

    pub fn disassemble(&self) -> String {
        #[allow(clippy::too_many_arguments)]
        fn push(
            text: &str,
            disassembly: &mut String,
            width: usize,
            indent: usize,
            center: bool,
            style_bold: bool,
            style_dim: bool,
            add_border: bool,
        ) {
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
            let length_before_content = disassembly.chars().count();

            for _ in 0..indent {
                disassembly.push_str("│   ");
            }

            if add_border {
                disassembly.push('│');
            }

            disassembly.push_str(&" ".repeat(left_pad_length));
            disassembly.push_str(&content);
            disassembly.push_str(&" ".repeat(right_pad_length));

            let length_after_content = disassembly.chars().count();
            let line_length = length_after_content - length_before_content;

            if line_length < content_width - 1 {
                disassembly.push_str(&" ".repeat(content_width - line_length));
            }

            if add_border {
                disassembly.push('│');
            }

            disassembly.push('\n');

            if !remainder.is_empty() {
                push(
                    remainder.iter().collect::<String>().as_str(),
                    disassembly,
                    width,
                    indent,
                    center,
                    style_bold,
                    style_dim,
                    add_border,
                );
            }
        }

        let push_header = |header: &str, disassembly: &mut String| {
            push(
                header,
                disassembly,
                self.width,
                self.indent,
                true,
                self.styled,
                false,
                true,
            );
        };
        let push_details = |details: &str, disassembly: &mut String| {
            push(
                details,
                disassembly,
                self.width,
                self.indent,
                true,
                false,
                false,
                true,
            );
        };
        let push_border = |border: &str, disassembly: &mut String| {
            push(
                border,
                disassembly,
                self.width,
                self.indent,
                false,
                false,
                false,
                false,
            )
        };
        let push_function_disassembly = |function_disassembly: &str, disassembly: &mut String| {
            disassembly.push_str(function_disassembly);
        };
        let mut disassembly = String::new();
        let top_border = "┌".to_string() + &"─".repeat(self.width - 2) + "┐";
        let section_border = "│".to_string() + &"┈".repeat(self.width - 2) + "│";
        let bottom_border = "└".to_string() + &"─".repeat(self.width - 2) + "┘";
        let name_display = self
            .chunk
            .name()
            .map(|identifier| identifier.to_string())
            .unwrap_or_else(|| {
                current_exe()
                    .map(|path| path.to_string_lossy().to_string())
                    .unwrap_or("Chunk Disassembly".to_string())
            });

        push_border(&top_border, &mut disassembly);
        push_header(&name_display, &mut disassembly);

        let info_line = format!(
            "{} instructions, {} constants, {} locals",
            self.chunk.instructions.len(),
            self.chunk.constants.len(),
            self.chunk.locals.len()
        );

        push(
            &info_line,
            &mut disassembly,
            self.width,
            self.indent,
            true,
            false,
            false,
            true,
        );

        for line in &Self::INSTRUCTION_HEADER {
            push_header(line, &mut disassembly);
        }

        for (index, (instruction, position)) in self.chunk.instructions.iter().enumerate() {
            let position = position.to_string();
            let operation = instruction.operation().to_string();
            let info = instruction.disassembly_info(Some(self.chunk));
            let bytecode = u32::from(instruction);
            let instruction_display =
                format!("{index:<5} {bytecode:<08X} {operation:15} {info:25} {position:13}");

            push_details(&instruction_display, &mut disassembly);
        }

        push_border(&section_border, &mut disassembly);

        for line in &Self::LOCAL_HEADER {
            push_header(line, &mut disassembly);
        }

        for (
            index,
            Local {
                identifier_index,
                r#type,
                scope: depth,
                register_index,
                is_mutable: mutable,
            },
        ) in self.chunk.locals.iter().enumerate()
        {
            let identifier_display = self
                .chunk
                .constants
                .get(*identifier_index as usize)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let type_display = r#type
                .as_ref()
                .map(|r#type| r#type.to_string())
                .unwrap_or("unknown".to_string());
            let local_display = format!(
                "{index:<5} {identifier_display:10} {type_display:8} {mutable:7} {depth:<5} {register_index:8}"
            );

            push_details(&local_display, &mut disassembly);
        }

        push_border(&section_border, &mut disassembly);

        for line in &Self::CONSTANT_HEADER {
            push_header(line, &mut disassembly);
        }

        for (index, value) in self.chunk.constants.iter().enumerate() {
            let constant_display = format!("{index:<5} {value:<5}");

            push_details(&constant_display, &mut disassembly);

            if let Some(function_disassembly) = match value {
                Value::Function(function) => Some({
                    let mut disassembler = function.chunk().disassembler();
                    disassembler.indent = self.indent + 1;

                    disassembler.styled(self.styled);
                    disassembler.disassemble()
                }),
                Value::Primitive(_) => None,
                Value::Object(_) => None,
            } {
                push_function_disassembly(&function_disassembly, &mut disassembly);
            }
        }

        push_border(&bottom_border, &mut disassembly);

        let _ = disassembly.trim_end_matches('\n');

        disassembly
    }
}
