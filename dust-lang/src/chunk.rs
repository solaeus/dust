use std::fmt::{self, Debug, Display, Formatter};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{AnnotatedError, Identifier, Instruction, Operation, Span, Value};

#[derive(Clone)]
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

    pub fn scope_depth(&self) -> usize {
        self.scope_depth
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
            .ok_or(ChunkError::CodeIndexOfBounds { offset, position })
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

    pub fn get_last_n_instructions<const N: usize>(&self) -> [Option<&(Instruction, Span)>; N] {
        let mut instructions = [None; N];

        for i in 0..N {
            let index = self.instructions.len().saturating_sub(i + 1);
            let instruction = self.instructions.get(index);

            instructions[i] = instruction;
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

        for i in 0..N {
            let index = self.instructions.len().saturating_sub(i + 1);

            let operation = self
                .instructions
                .get(index)
                .map(|(instruction, _)| instruction.operation());

            operations[i] = operation;
        }

        operations
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
        mutable: bool,
        register_index: u8,
        position: Span,
    ) -> Result<u8, ChunkError> {
        let starting_length = self.locals.len();

        if starting_length + 1 > (u8::MAX as usize) {
            Err(ChunkError::IdentifierOverflow { position })
        } else {
            self.locals.push(Local::new(
                identifier,
                mutable,
                self.scope_depth,
                Some(register_index),
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

        local.register_index = Some(register_index);

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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.disassembler("Chunk Display")
                .styled(true)
                .disassemble()
        )
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.disassembler("Chunk Debug Display")
                .styled(false)
                .disassemble()
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

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Local {
    pub identifier: Identifier,
    pub is_mutable: bool,
    pub depth: usize,
    pub register_index: Option<u8>,
}

impl Local {
    pub fn new(
        identifier: Identifier,
        mutable: bool,
        depth: usize,
        register_index: Option<u8>,
    ) -> Self {
        Self {
            identifier,
            is_mutable: mutable,
            depth,
            register_index,
        }
    }
}

pub struct ChunkDisassembler<'a> {
    name: &'a str,
    chunk: &'a Chunk,
    width: Option<usize>,
    styled: bool,
}

impl<'a> ChunkDisassembler<'a> {
    const INSTRUCTION_HEADER: [&'static str; 5] = [
        "",
        "Instructions",
        "------------",
        "INDEX BYTECODE OPERATION       INFO                      JUMP     POSITION",
        "----- -------- --------------- ------------------------- -------- --------",
    ];

    const CONSTANT_HEADER: [&'static str; 5] = [
        "",
        "Constants",
        "---------",
        "INDEX   VALUE  ",
        "----- ---------",
    ];

    const LOCAL_HEADER: [&'static str; 5] = [
        "",
        "Locals",
        "------",
        "INDEX IDENTIFIER MUTABLE DEPTH REGISTER",
        "----- ---------- ------- ----- --------",
    ];

    /// The default width of the disassembly output. To correctly align the output, this should
    /// return the width of the longest line that the disassembler is guaranteed to produce.
    pub fn default_width() -> usize {
        let longest_line = Self::INSTRUCTION_HEADER[4];

        longest_line.chars().count()
    }

    pub fn new(name: &'a str, chunk: &'a Chunk) -> Self {
        Self {
            name,
            chunk,
            width: None,
            styled: false,
        }
    }

    pub fn width(&mut self, width: usize) -> &mut Self {
        self.width = Some(width);

        self
    }

    pub fn styled(&mut self, styled: bool) -> &mut Self {
        self.styled = styled;

        self
    }

    pub fn disassemble(&self) -> String {
        let width = self.width.unwrap_or_else(Self::default_width);
        let center = |line: &str| format!("{line:^width$}\n");
        let style = |line: String| {
            if self.styled {
                line.bold().to_string()
            } else {
                line
            }
        };

        let mut disassembly = String::with_capacity(self.predict_length());
        let name_line = style(center(self.name));

        disassembly.push_str(&name_line);

        let info_line = center(&format!(
            "{} instructions, {} constants, {} locals",
            self.chunk.instructions.len(),
            self.chunk.constants.len(),
            self.chunk.locals.len()
        ));
        let styled_info_line = {
            if self.styled {
                info_line.dimmed().to_string()
            } else {
                info_line
            }
        };

        disassembly.push_str(&styled_info_line);

        for line in Self::INSTRUCTION_HEADER {
            disassembly.push_str(&style(center(line)));
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
                let jump_index = index + jump_offset + 1;

                format!("{index} -> {jump_index}")
            } else {
                " ".to_string()
            };
            let bytecode = u32::from(instruction);
            let instruction_display = format!(
                "{index:<5} {bytecode:<08X} {operation:15} {info:25} {jump_offset:8} {position:8}"
            );

            disassembly.push_str(&center(&instruction_display));
        }

        for line in Self::CONSTANT_HEADER {
            disassembly.push_str(&style(center(line)));
        }

        for (index, value_option) in self.chunk.constants.iter().enumerate() {
            let value_display = value_option
                .as_ref()
                .map(|value| value.to_string())
                .unwrap_or("empty".to_string());
            let trucated_length = 8;
            let with_elipsis = trucated_length - 3;
            let constant_display = if value_display.len() > with_elipsis {
                format!("{index:<5} {value_display:.<trucated_length$.with_elipsis$}")
            } else {
                format!("{index:<5} {value_display:<trucated_length$}")
            };

            disassembly.push_str(&center(&constant_display));
        }

        for line in Self::LOCAL_HEADER {
            disassembly.push_str(&style(center(line)));
        }

        for (
            index,
            Local {
                identifier,
                depth,
                register_index,
                is_mutable: mutable,
            },
        ) in self.chunk.locals.iter().enumerate()
        {
            let register_display = register_index
                .as_ref()
                .map(|value| value.to_string())
                .unwrap_or_else(|| "empty".to_string());
            let identifier_display = identifier.as_str();
            let local_display = format!(
                "{index:<5} {identifier_display:10} {mutable:7} {depth:<5} {register_display:8}"
            );

            disassembly.push_str(&center(&local_display));
        }

        let expected_length = self.predict_length();
        let actual_length = disassembly.len();

        if !self.styled && expected_length != actual_length {
            log::debug!(
                "Chunk disassembly was not optimized correctly, expected string length {expected_length}, got {actual_length}",
            );
        }

        if self.styled && expected_length > actual_length {
            log::debug!(
                "Chunk disassembly was not optimized correctly, expected string length to be at least{expected_length}, got {actual_length}",
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
        let width = self.width.unwrap_or_else(Self::default_width) + 1;

        total_line_count * width
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChunkError {
    CodeIndexOfBounds {
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
    IdentifierOverflow {
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
            ChunkError::CodeIndexOfBounds { .. } => "Code index out of bounds",
            ChunkError::ConstantAlreadyUsed { .. } => "Constant already used",
            ChunkError::ConstantOverflow { .. } => "Constant overflow",
            ChunkError::ConstantIndexOutOfBounds { .. } => "Constant index out of bounds",
            ChunkError::InstructionUnderflow { .. } => "Instruction underflow",
            ChunkError::LocalIndexOutOfBounds { .. } => "Identifier index out of bounds",
            ChunkError::IdentifierOverflow { .. } => "Identifier overflow",
            ChunkError::IdentifierNotFound { .. } => "Identifier not found",
        }
    }

    fn details(&self) -> Option<String> {
        match self {
            ChunkError::CodeIndexOfBounds { offset, .. } => Some(format!("Code index: {}", offset)),
            ChunkError::ConstantAlreadyUsed { index, .. } => {
                Some(format!("Constant index: {}", index))
            }
            ChunkError::ConstantIndexOutOfBounds { index, .. } => {
                Some(format!("Constant index: {}", index))
            }
            ChunkError::InstructionUnderflow { .. } => None,
            ChunkError::LocalIndexOutOfBounds { index, .. } => {
                Some(format!("Identifier index: {}", index))
            }
            ChunkError::IdentifierNotFound { identifier, .. } => {
                Some(format!("Identifier: {}", identifier))
            }
            ChunkError::IdentifierOverflow { .. } => Some("Identifier overflow".to_string()),
            ChunkError::ConstantOverflow { .. } => Some("Constant overflow".to_string()),
        }
    }

    fn position(&self) -> Span {
        match self {
            ChunkError::CodeIndexOfBounds { position, .. } => *position,
            ChunkError::ConstantAlreadyUsed { position, .. } => *position,
            ChunkError::ConstantIndexOutOfBounds { position, .. } => *position,
            ChunkError::IdentifierNotFound { position, .. } => *position,
            ChunkError::InstructionUnderflow { position, .. } => *position,
            ChunkError::LocalIndexOutOfBounds { position, .. } => *position,
            ChunkError::IdentifierOverflow { position, .. } => *position,
            ChunkError::ConstantOverflow { position, .. } => *position,
        }
    }
}
