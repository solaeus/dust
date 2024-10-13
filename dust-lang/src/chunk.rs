use std::fmt::{self, Debug, Display};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{AnnotatedError, Formatter, Identifier, Instruction, Operation, Span, Type, Value};

#[derive(Clone, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Chunk {
    instructions: Vec<(Instruction, Span)>,
    constants: Vec<Option<Value>>,
    locals: Vec<Local>,
    scope_depth: usize,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            locals: Vec::new(),
            scope_depth: 0,
        }
    }

    pub fn with_data(
        instructions: Vec<(Instruction, Span)>,
        constants: Vec<Value>,
        locals: Vec<Local>,
    ) -> Self {
        Self {
            instructions,
            constants: constants.into_iter().map(Some).collect(),
            locals,
            scope_depth: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    pub fn get_instruction(
        &self,
        offset: usize,
        position: Span,
    ) -> Result<&(Instruction, Span), ChunkError> {
        self.instructions
            .get(offset)
            .ok_or(ChunkError::InstructionIndexOfBounds { offset, position })
    }

    pub fn remove_instruction(&mut self, index: usize) -> (Instruction, Span) {
        self.instructions.remove(index)
    }

    pub fn push_instruction(&mut self, instruction: Instruction, position: Span) {
        self.instructions.push((instruction, position));
    }

    pub fn insert_instruction(&mut self, index: usize, instruction: Instruction, position: Span) {
        self.instructions.insert(index, (instruction, position));
    }

    pub fn pop_instruction(&mut self, position: Span) -> Result<(Instruction, Span), ChunkError> {
        self.instructions
            .pop()
            .ok_or(ChunkError::InstructionUnderflow { position })
    }

    pub fn get_last_instruction(&self) -> Result<(&Instruction, &Span), ChunkError> {
        let (instruction, position) =
            self.instructions
                .last()
                .ok_or_else(|| ChunkError::InstructionUnderflow {
                    position: Span(0, 0),
                })?;

        Ok((instruction, position))
    }

    pub fn get_last_n_instructions<const N: usize>(&self) -> [Option<(&Instruction, &Span)>; N] {
        let mut instructions = [None; N];

        for (index, (instruction, position)) in self.instructions.iter().rev().enumerate().take(N) {
            instructions[index] = Some((instruction, position));
        }

        instructions
    }

    pub fn find_last_instruction(&mut self, operation: Operation) -> Option<usize> {
        self.instructions
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, (instruction, _))| {
                if instruction.operation() == operation {
                    Some(index)
                } else {
                    None
                }
            })
    }

    pub fn get_last_operation(&self) -> Result<Operation, ChunkError> {
        self.get_last_instruction()
            .map(|(instruction, _)| instruction.operation())
    }

    pub fn get_last_n_operations<const N: usize>(&self) -> [Option<Operation>; N] {
        let mut operations = [None; N];

        for (index, (instruction, _)) in self.instructions.iter().rev().enumerate().take(N) {
            operations[index] = Some(instruction.operation());
        }

        operations
    }

    pub fn constants(&self) -> &[Option<Value>] {
        &self.constants
    }

    pub fn get_constant(&self, index: u8, position: Span) -> Result<&Value, ChunkError> {
        let index = index as usize;

        self.constants
            .get(index)
            .ok_or(ChunkError::ConstantIndexOutOfBounds { index, position })
            .and_then(|value| {
                value
                    .as_ref()
                    .ok_or(ChunkError::ConstantAlreadyUsed { index, position })
            })
    }

    pub fn take_constant(&mut self, index: u8, position: Span) -> Result<Value, ChunkError> {
        let index = index as usize;

        self.constants
            .get_mut(index)
            .ok_or_else(|| ChunkError::ConstantIndexOutOfBounds { index, position })?
            .take()
            .ok_or(ChunkError::ConstantAlreadyUsed { index, position })
    }

    pub fn push_constant(&mut self, value: Value, position: Span) -> Result<u8, ChunkError> {
        let starting_length = self.constants.len();

        if starting_length + 1 > (u8::MAX as usize) {
            Err(ChunkError::ConstantOverflow { position })
        } else {
            self.constants.push(Some(value));

            Ok(starting_length as u8)
        }
    }

    pub fn locals(&self) -> &[Local] {
        &self.locals
    }

    pub fn get_local(&self, index: u8, position: Span) -> Result<&Local, ChunkError> {
        let index = index as usize;

        self.locals
            .get(index)
            .ok_or(ChunkError::LocalIndexOutOfBounds { index, position })
    }

    pub fn get_identifier(&self, index: u8) -> Option<&Identifier> {
        self.locals
            .get(index as usize)
            .map(|local| &local.identifier)
    }

    pub fn get_local_index(
        &self,
        identifier: &Identifier,
        position: Span,
    ) -> Result<u8, ChunkError> {
        self.locals
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, local)| {
                if &local.identifier == identifier {
                    Some(index as u8)
                } else {
                    None
                }
            })
            .ok_or(ChunkError::IdentifierNotFound {
                identifier: identifier.clone(),
                position,
            })
    }

    pub fn declare_local(
        &mut self,
        identifier: Identifier,
        r#type: Option<Type>,
        is_mutable: bool,
        register_index: u8,
        position: Span,
    ) -> Result<u8, ChunkError> {
        log::debug!("Declare local {identifier}");

        let starting_length = self.locals.len();

        if starting_length + 1 > (u8::MAX as usize) {
            Err(ChunkError::LocalOverflow { position })
        } else {
            self.locals.push(Local::new(
                identifier,
                r#type,
                is_mutable,
                self.scope_depth,
                register_index,
            ));

            Ok(starting_length as u8)
        }
    }

    pub fn define_local(
        &mut self,
        local_index: u8,
        register_index: u8,
        position: Span,
    ) -> Result<(), ChunkError> {
        let local = self.locals.get_mut(local_index as usize).ok_or_else(|| {
            ChunkError::LocalIndexOutOfBounds {
                index: local_index as usize,
                position,
            }
        })?;

        log::debug!("Define local {}", local.identifier);

        local.register_index = register_index;

        Ok(())
    }

    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    pub fn end_scope(&mut self) {
        self.scope_depth -= 1;
    }

    pub fn disassembler<'a>(&'a self, name: &'a str) -> ChunkDisassembler<'a> {
        ChunkDisassembler::new(name, self)
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.disassembler("Chunk").styled(true).disassemble()
        )
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.disassembler("Chunk").styled(false).disassemble()
        )
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
    pub identifier: Identifier,
    pub r#type: Option<Type>,
    pub is_mutable: bool,
    pub depth: usize,
    pub register_index: u8,
}

impl Local {
    pub fn new(
        identifier: Identifier,
        r#type: Option<Type>,
        mutable: bool,
        depth: usize,
        register_index: u8,
    ) -> Self {
        Self {
            identifier,
            r#type,
            is_mutable: mutable,
            depth,
            register_index,
        }
    }
}

pub struct ChunkDisassembler<'a> {
    name: &'a str,
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
        "INDEX BYTECODE OPERATION       INFO                      JUMP     POSITION",
        "----- -------- --------------- ------------------------- -------- --------",
    ];

    const CONSTANT_HEADER: [&'static str; 4] =
        ["Constants", "---------", "INDEX VALUE", "----- -----"];

    const LOCAL_HEADER: [&'static str; 4] = [
        "Locals",
        "------",
        "INDEX IDENTIFIER TYPE     MUTABLE DEPTH REGISTER",
        "----- ---------- -------- ------- ----- --------",
    ];

    /// The default width of the disassembly output. To correctly align the output, this should
    /// return the width of the longest line that the disassembler is guaranteed to produce.
    pub fn default_width() -> usize {
        let longest_line = Self::INSTRUCTION_HEADER[3];

        longest_line.chars().count().max(80)
    }

    pub fn new(name: &'a str, chunk: &'a Chunk) -> Self {
        Self {
            name,
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

    pub fn indent(&mut self, indent: usize) -> &mut Self {
        self.indent = indent;

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

        push_border(&top_border, &mut disassembly);
        push_header(self.name, &mut disassembly);

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
            let (info, jump_offset) = instruction.disassembly_info(Some(self.chunk));
            let info = if let Some(info) = info {
                info
            } else {
                " ".to_string()
            };
            let jump_offset = if let Some(jump_offset) = jump_offset {
                let index = index as isize;
                let jump_index = {
                    if jump_offset > 0 {
                        index + (jump_offset + 1)
                    } else {
                        index + jump_offset
                    }
                };

                format!("{index} -> {jump_index}")
            } else {
                " ".to_string()
            };
            let bytecode = u32::from(instruction);
            let instruction_display = format!(
                "{index:<5} {bytecode:<08X} {operation:15} {info:25} {jump_offset:8} {position:8}"
            );

            push_details(&instruction_display, &mut disassembly);
        }

        push_border(&section_border, &mut disassembly);

        for line in &Self::LOCAL_HEADER {
            push_header(line, &mut disassembly);
        }

        for (
            index,
            Local {
                identifier,
                r#type,
                depth,
                register_index,
                is_mutable: mutable,
            },
        ) in self.chunk.locals.iter().enumerate()
        {
            let identifier_display = identifier.as_str();
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

        for (index, value_option) in self.chunk.constants.iter().enumerate() {
            let value_display = value_option
                .as_ref()
                .map(|value| value.to_string())
                .unwrap_or("empty".to_string());
            let constant_display = format!("{index:<5} {value_display:<5}");

            push_details(&constant_display, &mut disassembly);

            if let Some(function_disassembly) =
                value_option.as_ref().and_then(|value| match value {
                    Value::Function(function) => Some(
                        function
                            .chunk()
                            .disassembler("function")
                            .styled(self.styled)
                            .width(self.width)
                            .indent(self.indent + 1)
                            .disassemble(),
                    ),
                    Value::Primitive(_) => None,
                    Value::Object(_) => None,
                })
            {
                push_function_disassembly(&function_disassembly, &mut disassembly);
            }
        }

        push_border(&bottom_border, &mut disassembly);

        let expected_length = self.predict_length();
        let actual_length = disassembly.len();

        if !self.styled && expected_length != actual_length {
            log::debug!(
                "Chunk disassembly was not optimized correctly, expected string length {expected_length}, got {actual_length}",
            );
        }

        if self.styled && expected_length > actual_length {
            log::debug!(
                "Chunk disassembly was not optimized correctly, expected string length to be at least {expected_length}, got {actual_length}",
            );
        }

        disassembly
    }

    /// Predicts the capacity of the disassembled output. This is used to pre-allocate the string
    /// buffer to avoid reallocations.
    ///
    /// The capacity is calculated as follows:
    ///     - Get the number of static lines, i.e. lines that are always present in the disassembly
    ///     - Get the number of dynamic lines, i.e. lines that are generated from the chunk
    ///     - Add an one to the width to account for the newline character
    ///     - Multiply the total number of lines by the width of the disassembly output
    ///
    /// The result is accurate only if the output is not styled. Otherwise the extra bytes added by
    /// the ANSI escape codes will make the result too low. It still works as a lower bound in that
    /// case.
    fn predict_length(&self) -> usize {
        const EXTRA_LINES: usize = 2; // There is one info line and one empty line after the name

        let static_line_count = Self::INSTRUCTION_HEADER.len()
            + Self::CONSTANT_HEADER.len()
            + Self::LOCAL_HEADER.len()
            + EXTRA_LINES;
        let dynamic_line_count =
            self.chunk.instructions.len() + self.chunk.constants.len() + self.chunk.locals.len();
        let total_line_count = static_line_count + dynamic_line_count;

        total_line_count * self.width
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChunkError {
    InstructionIndexOfBounds {
        offset: usize,
        position: Span,
    },
    ConstantAlreadyUsed {
        index: usize,
        position: Span,
    },
    ConstantOverflow {
        position: Span,
    },
    ConstantIndexOutOfBounds {
        index: usize,
        position: Span,
    },
    InstructionUnderflow {
        position: Span,
    },
    LocalIndexOutOfBounds {
        index: usize,
        position: Span,
    },
    LocalOverflow {
        position: Span,
    },
    IdentifierNotFound {
        identifier: Identifier,
        position: Span,
    },
}

impl AnnotatedError for ChunkError {
    fn title() -> &'static str {
        "Chunk Error"
    }

    fn description(&self) -> &'static str {
        match self {
            ChunkError::InstructionIndexOfBounds { .. } => "Instruction index out of bounds",
            ChunkError::ConstantAlreadyUsed { .. } => "Constant already used",
            ChunkError::ConstantOverflow { .. } => "Constant overflow",
            ChunkError::ConstantIndexOutOfBounds { .. } => "Constant index out of bounds",
            ChunkError::InstructionUnderflow { .. } => "Instruction underflow",
            ChunkError::LocalIndexOutOfBounds { .. } => "Local index out of bounds",
            ChunkError::LocalOverflow { .. } => "Local overflow",
            ChunkError::IdentifierNotFound { .. } => "Identifier not found",
        }
    }

    fn details(&self) -> Option<String> {
        match self {
            ChunkError::InstructionIndexOfBounds { offset, .. } => {
                Some(format!("Instruction index: {}", offset))
            }
            ChunkError::ConstantAlreadyUsed { index, .. } => {
                Some(format!("Constant index: {}", index))
            }
            ChunkError::ConstantIndexOutOfBounds { index, .. } => {
                Some(format!("Constant index: {}", index))
            }
            ChunkError::InstructionUnderflow { .. } => None,
            ChunkError::LocalIndexOutOfBounds { index, .. } => {
                Some(format!("Local index: {}", index))
            }
            ChunkError::IdentifierNotFound { identifier, .. } => {
                Some(format!("Identifier: {}", identifier))
            }
            ChunkError::LocalOverflow { .. } => None,
            ChunkError::ConstantOverflow { .. } => None,
        }
    }

    fn position(&self) -> Span {
        match self {
            ChunkError::InstructionIndexOfBounds { position, .. } => *position,
            ChunkError::ConstantAlreadyUsed { position, .. } => *position,
            ChunkError::ConstantIndexOutOfBounds { position, .. } => *position,
            ChunkError::IdentifierNotFound { position, .. } => *position,
            ChunkError::InstructionUnderflow { position, .. } => *position,
            ChunkError::LocalIndexOutOfBounds { position, .. } => *position,
            ChunkError::LocalOverflow { position, .. } => *position,
            ChunkError::ConstantOverflow { position, .. } => *position,
        }
    }
}
