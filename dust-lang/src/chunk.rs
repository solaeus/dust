use std::fmt::{self, Debug, Display, Formatter};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{AnnotatedError, Identifier, Instruction, Span, Value};

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

    pub fn get_instruction(
        &self,
        offset: usize,
        position: Span,
    ) -> Result<&(Instruction, Span), ChunkError> {
        self.instructions
            .get(offset)
            .ok_or(ChunkError::CodeIndexOfBounds { offset, position })
    }

    pub fn push_instruction(&mut self, instruction: Instruction, position: Span) {
        self.instructions.push((instruction, position));
    }

    pub fn pop_instruction(&mut self, position: Span) -> Result<(Instruction, Span), ChunkError> {
        self.instructions
            .pop()
            .ok_or(ChunkError::InstructionUnderflow { position })
    }

    pub fn get_constant(&self, index: usize, position: Span) -> Result<&Value, ChunkError> {
        self.constants
            .get(index)
            .ok_or(ChunkError::ConstantIndexOutOfBounds { index, position })
            .and_then(|value| {
                value
                    .as_ref()
                    .ok_or(ChunkError::ConstantAlreadyUsed { index, position })
            })
    }

    pub fn take_constant(&mut self, index: usize, position: Span) -> Result<Value, ChunkError> {
        self.constants
            .get_mut(index)
            .ok_or_else(|| ChunkError::ConstantIndexOutOfBounds { index, position })?
            .take()
            .ok_or(ChunkError::ConstantAlreadyUsed { index, position })
    }

    pub fn push_constant(&mut self, value: Value, position: Span) -> Result<u16, ChunkError> {
        let starting_length = self.constants.len();

        if starting_length + 1 > (u8::MAX as usize) {
            Err(ChunkError::ConstantOverflow { position })
        } else {
            self.constants.push(Some(value));

            Ok(starting_length as u16)
        }
    }

    pub fn get_local(&self, index: usize, position: Span) -> Result<&Local, ChunkError> {
        self.locals
            .get(index)
            .ok_or(ChunkError::LocalIndexOutOfBounds { index, position })
    }

    pub fn get_identifier(&self, index: usize) -> Option<&Identifier> {
        self.locals.get(index).map(|local| &local.identifier)
    }

    pub fn get_local_index(
        &self,
        identifier: &Identifier,
        position: Span,
    ) -> Result<u16, ChunkError> {
        self.locals
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, local)| {
                if &local.identifier == identifier {
                    Some(index as u16)
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
        position: Span,
    ) -> Result<u16, ChunkError> {
        let starting_length = self.locals.len();

        if starting_length + 1 > (u8::MAX as usize) {
            Err(ChunkError::IdentifierOverflow { position })
        } else {
            self.locals
                .push(Local::new(identifier, self.scope_depth, None));

            Ok(starting_length as u16)
        }
    }

    pub fn define_local(
        &mut self,
        local_index: usize,
        register_index: u8,
        position: Span,
    ) -> Result<(), ChunkError> {
        let local =
            self.locals
                .get_mut(local_index)
                .ok_or_else(|| ChunkError::LocalIndexOutOfBounds {
                    index: local_index,
                    position,
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
    pub depth: usize,
    pub register_index: Option<u8>,
}

impl Local {
    pub fn new(identifier: Identifier, depth: usize, register_index: Option<u8>) -> Self {
        Self {
            identifier,
            depth,
            register_index,
        }
    }
}

pub struct ChunkDisassembler<'a> {
    name: &'a str,
    chunk: &'a Chunk,
    width: usize,
    styled: bool,
}

impl<'a> ChunkDisassembler<'a> {
    const INSTRUCTION_HEADER: [&'static str; 5] = [
        "",
        "Instructions",
        "------------",
        "OFFSET  OPERATION      INFO                 POSITION",
        "------- -------------- -------------------- --------",
    ];

    const CONSTANT_HEADER: [&'static str; 5] = [
        "",
        "Constants",
        "---------",
        "INDEX KIND  VALUE",
        "----- ----- -----",
    ];

    const LOCAL_HEADER: [&'static str; 5] = [
        "",
        "Locals",
        "------",
        "INDEX IDENTIFIER DEPTH REGISTER",
        "----- ---------- ----- --------",
    ];

    /// The default width of the disassembly output. To correctly align the output, this should be
    /// set to the width of the longest line that the disassembler is guaranteed to produce.
    const DEFAULT_WIDTH: usize = Self::INSTRUCTION_HEADER[4].len() + 1;

    pub fn new(name: &'a str, chunk: &'a Chunk) -> Self {
        Self {
            name,
            chunk,
            width: Self::DEFAULT_WIDTH,
            styled: false,
        }
    }

    pub fn disassemble(&self) -> String {
        let center = |line: &str| format!("{line:^width$}\n", width = self.width);
        let style = |line: String| {
            if self.styled {
                line.bold().to_string()
            } else {
                line
            }
        };

        let mut disassembled = String::with_capacity(self.predict_length());
        let name_line = style(center(self.name));

        disassembled.push_str(&name_line);

        for line in Self::INSTRUCTION_HEADER {
            disassembled.push_str(&style(center(line)));
        }

        for (offset, (instruction, position)) in self.chunk.instructions.iter().enumerate() {
            let position = position.to_string();
            let operation = instruction.operation.to_string();
            let info_option = instruction.disassembly_info(Some(self.chunk));
            let instruction_display = if let Some(info) = info_option {
                format!("{offset:<7} {operation:14} {info:20} {position:8}")
            } else {
                format!("{offset:<7} {operation:14} {:20} {position:8}", " ")
            };

            disassembled.push_str(&center(&instruction_display));
        }

        for line in Self::CONSTANT_HEADER {
            disassembled.push_str(&style(center(line)));
        }

        for (index, value_option) in self.chunk.constants.iter().enumerate() {
            let value_kind_display = if let Some(value) = value_option {
                value.kind().to_string()
            } else {
                "empty".to_string()
            };
            let value_display = value_option
                .as_ref()
                .map(|value| value.to_string())
                .unwrap_or_else(|| "empty".to_string());
            let constant_display = format!("{index:<5} {value_kind_display:<5} {value_display:<5}");

            disassembled.push_str(&center(&constant_display));
        }

        for line in Self::LOCAL_HEADER {
            disassembled.push_str(&style(center(line)));
        }

        for (
            index,
            Local {
                identifier,
                depth,
                register_index,
            },
        ) in self.chunk.locals.iter().enumerate()
        {
            let register_display = register_index
                .as_ref()
                .map(|value| value.to_string())
                .unwrap_or_else(|| "empty".to_string());
            let identifier_display = identifier.as_str();
            let local_display =
                format!("{index:<5} {identifier_display:<10} {depth:<5} {register_display:<8}");

            disassembled.push_str(&center(&local_display));
        }

        let expected_length = self.predict_length();
        let actual_length = disassembled.len();

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

        disassembled
    }

    pub fn width(&mut self, width: usize) -> &mut Self {
        self.width = width;

        self
    }

    pub fn styled(&mut self, styled: bool) -> &mut Self {
        self.styled = styled;

        self
    }

    /// Predicts the capacity of the disassembled output. This is used to pre-allocate the string
    /// buffer to avoid reallocations.
    ///
    /// The capacity is calculated as follows:
    ///     - Get the number of static lines, i.e. lines that are always present in the disassembly
    ///     - Get the number of dynamic lines, i.e. lines that are generated from the chunk
    ///     - Add 1 to the width to account for the newline character
    ///     - Multiply the total number of lines by the width of the disassembly output
    ///
    /// The result is accurate only if the output is not styled. Otherwise the extra bytes added by
    /// the ANSI escape codes will make the result too low. It still works as a lower bound in that
    /// case.
    fn predict_length(&self) -> usize {
        const EXTRA_LINES: usize = 1; // There is one empty line after the name of the chunk

        let static_line_count =
            Self::INSTRUCTION_HEADER.len() + Self::CONSTANT_HEADER.len() + Self::LOCAL_HEADER.len();
        let dynamic_line_count =
            self.chunk.instructions.len() + self.chunk.constants.len() + self.chunk.locals.len();
        let total_line_count = static_line_count + dynamic_line_count + EXTRA_LINES;

        total_line_count * (self.width + 1)
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
            _ => None,
        }
    }

    fn position(&self) -> Span {
        match self {
            ChunkError::CodeIndexOfBounds { position, .. } => *position,
            ChunkError::ConstantAlreadyUsed { position, .. } => *position,
            ChunkError::ConstantIndexOutOfBounds { position, .. } => *position,
            ChunkError::IdentifierNotFound { position, .. } => *position,
            _ => todo!(),
        }
    }
}
