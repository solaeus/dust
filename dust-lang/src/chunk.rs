//! In-memory representation of a Dust program or function.
//!
//! A chunk consists of a sequence of instructions and their positions, a list of constants, and a
//! list of locals that can be executed by the Dust virtual machine. Chunks have a name when they
//! belong to a named function.
//!
//! # Disassembly
//!
//! Chunks can be disassembled into a human-readable format using the `disassemble` method. The
//! output is designed to be displayed in a terminal and is styled for readability.

use std::{
    cmp::Ordering,
    env::current_exe,
    fmt::{self, Debug, Display},
};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{Instruction, Span, Type, Value};

/// In-memory representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Chunk {
    name: Option<String>,

    instructions: Vec<(Instruction, Span)>,
    constants: Vec<Value>,
    locals: Vec<Local>,

    current_scope: Scope,
    scope_index: u8,
}

impl Chunk {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            instructions: Vec::new(),
            constants: Vec::new(),
            locals: Vec::new(),
            current_scope: Scope::default(),
            scope_index: 0,
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
            scope_index: 0,
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
        self.scope_index += 1;
        self.current_scope.index = self.scope_index;
        self.current_scope.depth += 1;
    }

    pub fn end_scope(&mut self) {
        self.current_scope.depth -= 1;

        if self.current_scope.depth == 0 {
            self.current_scope.index = 0;
        } else {
            self.current_scope.index -= 1;
        }
    }

    pub fn get_constant_type(&self, constant_index: u8) -> Option<Type> {
        self.constants
            .get(constant_index as usize)
            .map(|value| value.r#type())
    }

    pub fn get_local_type(&self, local_index: u8) -> Option<Type> {
        self.locals.get(local_index as usize)?.r#type.clone()
    }

    pub fn get_register_type(&self, register_index: u8) -> Option<Type> {
        let local_type_option = self
            .locals
            .iter()
            .find(|local| local.register_index == register_index)
            .map(|local| local.r#type.clone());

        if let Some(local_type) = local_type_option {
            return local_type;
        }

        self.instructions.iter().find_map(|(instruction, _)| {
            if instruction.yields_value() && instruction.a() == register_index {
                instruction.yielded_type(self)
            } else {
                None
            }
        })
    }

    pub fn return_type(&self) -> Option<Type> {
        self.instructions.iter().rev().find_map(|(instruction, _)| {
            if instruction.yields_value() {
                instruction.yielded_type(self)
            } else {
                None
            }
        })
    }

    pub fn disassembler(&self) -> ChunkDisassembler {
        ChunkDisassembler::new(self)
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disassembler = self.disassembler().styled(false);

        write!(f, "{}", disassembler.disassemble())
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
    pub depth: u8,
    /// The nth scope in the chunk.
    pub index: u8,
}

impl Scope {
    pub fn new(index: u8, width: u8) -> Self {
        Self {
            depth: index,
            index: width,
        }
    }

    pub fn contains(&self, other: &Self) -> bool {
        match self.depth.cmp(&other.depth) {
            Ordering::Less => false,
            Ordering::Greater => self.index >= other.index,
            Ordering::Equal => self.index == other.index,
        }
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.depth, self.index)
    }
}

pub struct ChunkDisassembler<'a> {
    output: String,
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
        "INDEX BYTECODE OPERATION     INFO                      TYPE      POSITION   ",
        "----- -------- ------------- ------------------------- --------- -----------",
    ];

    const CONSTANT_HEADER: [&'static str; 4] =
        ["Constants", "---------", "INDEX VALUE", "----- -----"];

    const LOCAL_HEADER: [&'static str; 4] = [
        "Locals",
        "------",
        "INDEX IDENTIFIER TYPE     MUTABLE SCOPE   REGISTER",
        "----- ---------- -------- ------- ------- --------",
    ];

    /// The default width of the disassembly output. To correctly align the output, this should
    /// return the width of the longest line that the disassembler is guaranteed to produce.
    pub fn default_width() -> usize {
        let longest_line = Self::INSTRUCTION_HEADER[3];

        longest_line.chars().count().max(80)
    }

    pub fn new(chunk: &'a Chunk) -> Self {
        Self {
            output: String::new(),
            chunk,
            source: None,
            width: Self::default_width(),
            styled: false,
            indent: 0,
        }
    }

    pub fn source(mut self, source: &'a str) -> Self {
        self.source = Some(source);

        self
    }

    pub fn width(mut self, width: usize) -> Self {
        self.width = width;

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
        let characters = text.chars().collect::<Vec<char>>();
        let content_width = if add_border {
            self.width - 2
        } else {
            self.width
        };
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

    pub fn disassemble(mut self) -> String {
        let top_border = "┌".to_string() + &"─".repeat(self.width - 2) + "┐";
        let section_border = "│".to_string() + &"┈".repeat(self.width - 2) + "│";
        let bottom_border = "└".to_string() + &"─".repeat(self.width - 2) + "┘";
        let name_display = self
            .chunk
            .name
            .as_ref()
            .map(|identifier| identifier.to_string())
            .unwrap_or_else(|| {
                current_exe()
                    .map(|path| path.to_string_lossy().to_string())
                    .unwrap_or("Chunk Disassembly".to_string())
            });

        self.push_border(&top_border);
        self.push_header(&name_display);

        let info_line = format!(
            "{} instructions, {} constants, {} locals, returns {}",
            self.chunk.instructions.len(),
            self.chunk.constants.len(),
            self.chunk.locals.len(),
            self.chunk
                .return_type()
                .map(|r#type| r#type.to_string())
                .unwrap_or("none".to_string())
        );

        self.push(&info_line, true, false, false, true);

        for line in &Self::INSTRUCTION_HEADER {
            self.push_header(line);
        }

        for (index, (instruction, position)) in self.chunk.instructions.iter().enumerate() {
            let bytecode = u32::from(instruction);
            let operation = instruction.operation().to_string();
            let info = instruction.disassembly_info(self.chunk);
            let type_display = instruction
                .yielded_type(self.chunk)
                .map(|r#type| r#type.to_string())
                .unwrap_or(String::with_capacity(0));
            let position = position.to_string();

            let instruction_display = format!(
                "{index:<5} {bytecode:08X} {operation:13} {info:25} {type_display:9} {position:11}"
            );

            self.push_details(&instruction_display);
        }

        self.push_border(&section_border);

        for line in &Self::LOCAL_HEADER {
            self.push_header(line);
        }

        for (
            index,
            Local {
                identifier_index,
                r#type,
                scope,
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
                "{index:<5} {identifier_display:10} {type_display:8} {mutable:7} {scope:7} {register_index:8}"
            );

            self.push_details(&local_display);
        }

        self.push_border(&section_border);

        for line in &Self::CONSTANT_HEADER {
            self.push_header(line);
        }

        for (index, value) in self.chunk.constants.iter().enumerate() {
            let constant_display = format!("{index:<5} {value:<5}");

            self.push_details(&constant_display);

            if let Some(function_disassembly) = match value {
                Value::Function(function) => Some({
                    function
                        .chunk()
                        .disassembler()
                        .styled(self.styled)
                        .indent(self.indent + 1)
                        .disassemble()
                }),
                Value::Primitive(_) => None,
                Value::Object(_) => None,
            } {
                self.output.push_str(&function_disassembly);
            }
        }

        self.push_border(&bottom_border);

        let _ = self.output.trim_end_matches('\n');

        self.output
    }
}
