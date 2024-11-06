//! In-memory representation of a Dust program or function.
//!
//! A chunk consists of a sequence of instructions and their positions, a list of constants, and a
//! list of locals that can be executed by the Dust virtual machine. Chunks have a name when they
//! belong to a named function.
//!
//! # Disassembly
//!
//! Chunks can be disassembled into a human-readable format using the `disassemble` method. The
//! output is designed to be displayed in a terminal and can be styled for better readability.
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────────────────┐
//! │           /var/home/jeff/Repositories/dust/target/debug/dust-shell           │
//! │             3 instructions, 1 constants, 0 locals, returns none              │
//! │                                 Instructions                                 │
//! │                                 ------------                                 │
//! │ INDEX BYTECODE OPERATION     INFO                      TYPE      POSITION    │
//! │ ----- -------- ------------- ------------------------- --------- ----------- │
//! │ 0     00000003 LOAD_CONSTANT R0 = C0                   str       (11, 26)    │
//! │ 1     01390117 CALL_NATIVE   write_line(R0)                      (0, 27)     │
//! │ 2     00000018 RETURN                                            (27, 27)    │
//! │┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈│
//! │                                    Locals                                    │
//! │                                    ------                                    │
//! │              INDEX IDENTIFIER TYPE     MUTABLE SCOPE   REGISTER              │
//! │              ----- ---------- -------- ------- ------- --------              │
//! │┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈│
//! │                                  Constants                                   │
//! │                                  ---------                                   │
//! │                            INDEX      VALUE                                  │
//! │                            ----- ---------------                             │
//! │                            0      Hello, world!                              │
//! └──────────────────────────────────────────────────────────────────────────────┘
//! ```

use std::{
    cmp::Ordering,
    env::current_exe,
    fmt::{self, Debug, Display, Formatter},
};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{ConcreteValue, Instruction, Operation, Span, Type, Value};

/// In-memory representation of a Dust program or function.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Chunk {
    name: Option<String>,
    pub is_poisoned: bool,

    instructions: Vec<(Instruction, Span)>,
    constants: Vec<Value>,
    locals: Vec<Local>,

    current_scope: Scope,
    block_index: u8,
}

impl Chunk {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            is_poisoned: false,
            instructions: Vec::new(),
            constants: Vec::new(),
            locals: Vec::new(),
            current_scope: Scope::default(),
            block_index: 0,
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
            is_poisoned: false,
            instructions,
            constants,
            locals,
            current_scope: Scope::default(),
            block_index: 0,
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

    pub fn get_instruction(&self, index: usize) -> Result<&(Instruction, Span), ChunkError> {
        self.instructions
            .get(index)
            .ok_or(ChunkError::InstructionIndexOutOfBounds { index })
    }

    pub fn locals(&self) -> &Vec<Local> {
        &self.locals
    }

    pub fn locals_mut(&mut self) -> &mut Vec<Local> {
        &mut self.locals
    }

    pub fn get_local(&self, index: u8) -> Result<&Local, ChunkError> {
        self.locals
            .get(index as usize)
            .ok_or(ChunkError::LocalIndexOutOfBounds {
                index: index as usize,
            })
    }

    pub fn get_local_mut(&mut self, index: u8) -> Result<&mut Local, ChunkError> {
        self.locals
            .get_mut(index as usize)
            .ok_or(ChunkError::LocalIndexOutOfBounds {
                index: index as usize,
            })
    }

    pub fn current_scope(&self) -> Scope {
        self.current_scope
    }

    pub fn get_constant(&self, index: u8) -> Result<&Value, ChunkError> {
        self.constants
            .get(index as usize)
            .ok_or(ChunkError::ConstantIndexOutOfBounds {
                index: index as usize,
            })
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
        self.block_index += 1;
        self.current_scope.block_index = self.block_index;
        self.current_scope.depth += 1;
    }

    pub fn end_scope(&mut self) {
        self.current_scope.depth -= 1;

        if self.current_scope.depth == 0 {
            self.current_scope.block_index = 0;
        } else {
            self.current_scope.block_index -= 1;
        }
    }

    pub fn expect_not_poisoned(&self) -> Result<(), ChunkError> {
        if self.is_poisoned {
            Err(ChunkError::PoisonedChunk)
        } else {
            Ok(())
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

        self.instructions
            .iter()
            .enumerate()
            .find_map(|(index, (instruction, _))| {
                if let Operation::LoadList = instruction.operation() {
                    if instruction.a() == register_index {
                        let mut length = (instruction.c() - instruction.b() + 1) as usize;
                        let mut item_type = Type::Any;
                        let distance_to_end = self.len() - index;

                        for (instruction, _) in self
                            .instructions()
                            .iter()
                            .rev()
                            .skip(distance_to_end)
                            .take(length)
                        {
                            if let Operation::Close = instruction.operation() {
                                length -= (instruction.c() - instruction.b()) as usize;
                            } else if let Type::Any = item_type {
                                item_type = instruction.yielded_type(self).unwrap_or(Type::Any);
                            }
                        }

                        return Some(Type::List {
                            item_type: Box::new(item_type),
                            length,
                        });
                    }
                }

                if instruction.yields_value() && instruction.a() == register_index {
                    instruction.yielded_type(self)
                } else {
                    None
                }
            })
    }

    pub fn return_type(&self) -> Option<Type> {
        let returns_value = self
            .instructions()
            .last()
            .map(|(instruction, _)| {
                debug_assert!(matches!(instruction.operation(), Operation::Return));

                instruction.b_as_boolean()
            })
            .unwrap_or(false);

        if returns_value {
            self.instructions.iter().rev().find_map(|(instruction, _)| {
                if instruction.yields_value() {
                    instruction.yielded_type(self)
                } else {
                    None
                }
            })
        } else {
            None
        }
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

/// A scoped variable.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Local {
    /// The index of the identifier in the constants table.
    pub identifier_index: u8,

    /// The expected type of the local's value.
    pub r#type: Option<Type>,

    /// Whether the local is mutable.
    pub is_mutable: bool,

    /// Scope where the variable was declared.
    pub scope: Scope,

    /// Expected location of a local's value.
    pub register_index: u8,
}

impl Local {
    /// Creates a new Local instance.
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

/// Variable locality, as defined by its depth and block index.
///
/// The `block index` is a unique identifier for a block within a chunk. It is used to differentiate
/// between blocks that are not nested together but have the same depth, i.e. sibling scopes. If the
/// `block_index` is 0, then the scope is the root scope of the chunk. The `block_index` is always 0
/// when the `depth` is 0. See [Chunk::begin_scope][] and [Chunk::end_scope][] to see how scopes are
/// incremented and decremented.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Scope {
    /// Level of block nesting.
    pub depth: u8,
    /// Index of the block in the chunk.
    pub block_index: u8,
}

impl Scope {
    pub fn new(depth: u8, block_index: u8) -> Self {
        Self { depth, block_index }
    }

    pub fn contains(&self, other: &Self) -> bool {
        match self.depth.cmp(&other.depth) {
            Ordering::Less => false,
            Ordering::Greater => self.block_index >= other.block_index,
            Ordering::Equal => self.block_index == other.block_index,
        }
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.depth, self.block_index)
    }
}

/// Builder that constructs a human-readable representation of a chunk.
pub struct ChunkDisassembler<'a> {
    output: String,
    chunk: &'a Chunk,
    source: Option<&'a str>,

    // Options
    width: usize,
    styled: bool,
    indent: usize,
}

impl<'a> ChunkDisassembler<'a> {
    const INSTRUCTION_HEADER: [&'static str; 4] = [
        "Instructions",
        "------------",
        " i  BYTECODE OPERATION             INFO               TYPE        POSITION  ",
        "--- -------- ------------- -------------------- --------------- ------------",
    ];

    const CONSTANT_HEADER: [&'static str; 4] = [
        "Constants",
        "---------",
        "INDEX      VALUE     ",
        "----- ---------------",
    ];

    const LOCAL_HEADER: [&'static str; 4] = [
        "Locals",
        "------",
        "INDEX IDENTIFIER TYPE       MUTABLE SCOPE   REGISTER",
        "----- ---------- ---------- ------- ------- --------",
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

    fn push_empty(&mut self) {
        self.push("", false, false, false, true);
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
            self.chunk.instructions.len(),
            self.chunk.constants.len(),
            self.chunk.locals.len(),
            self.chunk
                .return_type()
                .map(|r#type| r#type.to_string())
                .unwrap_or("none".to_string())
        );

        self.push(&info_line, true, false, true, true);
        self.push_empty();

        for line in &Self::INSTRUCTION_HEADER {
            self.push_header(line);
        }

        for (index, (instruction, position)) in self.chunk.instructions.iter().enumerate() {
            let bytecode = format!("{:02X}", u32::from(instruction));
            let operation = instruction.operation().to_string();
            let info = instruction.disassembly_info(self.chunk);
            let type_display = instruction
                .yielded_type(self.chunk)
                .map(|r#type| {
                    let type_string = r#type.to_string();

                    if type_string.len() > 15 {
                        format!("{type_string:.12}...")
                    } else {
                        type_string
                    }
                })
                .unwrap_or(String::with_capacity(0));
            let position = position.to_string();

            let instruction_display = format!(
                "{index:^3} {bytecode:>8} {operation:13} {info:20} {type_display:^15} {position:12}"
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
                .map(|r#type| {
                    let type_string = r#type.to_string();

                    if type_string.len() > 10 {
                        format!("{type_string:.7}...")
                    } else {
                        type_string
                    }
                })
                .unwrap_or("unknown".to_string());
            let local_display = format!(
                "{index:<5} {identifier_display:10} {type_display:10} {mutable:7} {scope:7} {register_index:8}"
            );

            self.push_details(&local_display);
        }

        self.push_border(&section_border);

        for line in &Self::CONSTANT_HEADER {
            self.push_header(line);
        }

        for (index, value) in self.chunk.constants.iter().enumerate() {
            let value_display = {
                let value_string = value.to_string();

                if value_string.len() > 15 {
                    format!("{value_string:.12}...")
                } else {
                    value_string
                }
            };
            let constant_display = format!("{index:<5} {value_display:^15}");

            self.push_details(&constant_display);

            if let Some(function_disassembly) = match value {
                Value::Concrete(ConcreteValue::Function(function)) => Some({
                    function
                        .chunk()
                        .disassembler()
                        .styled(self.styled)
                        .indent(self.indent + 1)
                        .disassemble()
                }),
                _ => None,
            } {
                self.output.push_str(&function_disassembly);
            }
        }

        self.push_border(&bottom_border);

        let _ = self.output.trim_end_matches('\n');

        self.output
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ChunkError {
    ConstantIndexOutOfBounds { index: usize },
    InstructionIndexOutOfBounds { index: usize },
    LocalIndexOutOfBounds { index: usize },
    PoisonedChunk,
}

impl Display for ChunkError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ChunkError::ConstantIndexOutOfBounds { index } => {
                write!(f, "Constant index {} out of bounds", index)
            }
            ChunkError::InstructionIndexOutOfBounds { index } => {
                write!(f, "Instruction index {} out of bounds", index)
            }
            ChunkError::LocalIndexOutOfBounds { index } => {
                write!(f, "Local index {} out of bounds", index)
            }
            ChunkError::PoisonedChunk => write!(f, "Chunk is poisoned"),
        }
    }
}
