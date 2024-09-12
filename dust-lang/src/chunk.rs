use std::fmt::{self, Debug, Display, Formatter};

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
        identifiers: Vec<Local>,
    ) -> Self {
        Self {
            instructions,
            constants: constants.into_iter().map(Some).collect(),
            locals: identifiers,
            scope_depth: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
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

    pub fn pop_instruction(&mut self) -> Option<(Instruction, Span)> {
        self.instructions.pop()
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

    pub fn use_constant(&mut self, index: usize, position: Span) -> Result<Value, ChunkError> {
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

    pub fn contains_identifier(&self, identifier: &Identifier) -> bool {
        self.locals
            .iter()
            .any(|local| &local.identifier == identifier)
    }

    pub fn get_local(&self, index: usize, position: Span) -> Result<&Local, ChunkError> {
        self.locals
            .get(index)
            .ok_or(ChunkError::LocalIndexOutOfBounds { index, position })
    }

    pub fn get_identifier(&self, index: u8) -> Option<&Identifier> {
        if let Some(local) = self.locals.get(index as usize) {
            Some(&local.identifier)
        } else {
            None
        }
    }

    pub fn get_local_index(
        &self,
        identifier: &Identifier,
        position: Span,
    ) -> Result<u16, ChunkError> {
        self.locals
            .iter()
            .rev()
            .enumerate()
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
        index: usize,
        value: Value,
        position: Span,
    ) -> Result<(), ChunkError> {
        let local = self
            .locals
            .get_mut(index)
            .ok_or_else(|| ChunkError::LocalIndexOutOfBounds { index, position })?;

        local.value = Some(value);

        Ok(())
    }

    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    pub fn end_scope(&mut self) {
        self.scope_depth -= 1;
    }

    pub fn clear(&mut self) {
        self.instructions.clear();
        self.constants.clear();
        self.locals.clear();
    }

    pub fn identifiers(&self) -> &[Local] {
        &self.locals
    }

    pub fn pop_identifier(&mut self) -> Option<Local> {
        self.locals.pop()
    }

    pub fn disassemble<'a>(&self, name: &'a str) -> DisassembledChunk<'a> {
        DisassembledChunk::new(name, self)
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.disassemble("Chunk Display"))
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.disassemble("Chunk Debug Display"))
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
    pub value: Option<Value>,
}

impl Local {
    pub fn new(identifier: Identifier, depth: usize, value: Option<Value>) -> Self {
        Self {
            identifier,
            depth,
            value,
        }
    }
}

pub struct DisassembledChunk<'a> {
    name: &'a str,
    body: String,
}

impl DisassembledChunk<'_> {
    pub fn new<'a>(name: &'a str, chunk: &Chunk) -> DisassembledChunk<'a> {
        let mut disassembled = String::new();
        let mut longest_line = 0;

        disassembled.push('\n');
        disassembled.push_str(&DisassembledChunk::instructions_header());
        disassembled.push('\n');

        for (offset, (instruction, position)) in chunk.instructions.iter().enumerate() {
            let position = position.to_string();
            let operation = instruction.operation.to_string();
            let info_option = instruction.disassembly_info(Some(chunk));
            let instruction_display = if let Some(info) = info_option {
                format!("{offset:<6} {operation:16} {info:17} {position:8}\n")
            } else {
                format!("{offset:<6} {operation:16} {:17} {position:8}\n", " ")
            };

            disassembled.push_str(&instruction_display);

            let line_length = instruction_display.len();

            if line_length > longest_line {
                longest_line = line_length;
            }
        }

        let mut push_centered = |section: &str| {
            let mut centered = String::new();

            for line in section.lines() {
                centered.push_str(&format!("{line:^longest_line$}\n"));
            }

            disassembled.push_str(&centered);
        };

        push_centered("\n");
        push_centered(&mut DisassembledChunk::constant_header());

        for (index, value_option) in chunk.constants.iter().enumerate() {
            let value_kind_display = match value_option {
                Some(Value::Raw(_)) => "RAW ",
                Some(Value::Reference(_)) => "REF ",
                Some(Value::Mutable(_)) => "MUT ",
                None => "EMPTY",
            };
            let value_display = value_option
                .as_ref()
                .map(|value| value.to_string())
                .unwrap_or_else(|| "EMPTY".to_string());
            let constant_display = format!("{index:<5} {value_kind_display:<4} {value_display:<5}");

            push_centered(&constant_display);
        }

        push_centered("\n");
        push_centered(&mut DisassembledChunk::local_header());

        for (
            index,
            Local {
                identifier,
                depth,
                value,
            },
        ) in chunk.locals.iter().enumerate()
        {
            let value_kind_display = match value {
                Some(Value::Raw(_)) => "RAW ",
                Some(Value::Reference(_)) => "REF ",
                Some(Value::Mutable(_)) => "MUT ",
                None => "EMPTY",
            };
            let value_display = value
                .as_ref()
                .map(|value| value.to_string())
                .unwrap_or_else(|| "EMPTY".to_string());
            let identifier_display = identifier.as_str();
            let local_display =
                format!("{index:<5} {identifier_display:<10} {depth:<5} {value_kind_display:<4} {value_display:<5}");

            push_centered(&local_display);
        }

        DisassembledChunk {
            name,
            body: disassembled,
        }
    }

    pub fn to_string_with_width(&self, width: usize) -> String {
        let mut display = String::new();

        for line in self.to_string().lines() {
            display.push_str(&format!("{line:^width$}\n"));
        }

        display
    }

    fn name_header(&self) -> String {
        format!("{:^50}\n{:^50}", self.name, "==============")
    }

    fn instructions_header() -> String {
        format!(
            "{:^50}\n{:^50}\n{:<6} {:<16} {:<17} {}\n{} {} {} {}",
            "Instructions",
            "------------",
            "OFFSET",
            "INSTRUCTION",
            "INFO",
            "POSITION",
            "------",
            "----------------",
            "-----------------",
            "--------"
        )
    }

    fn constant_header() -> String {
        format!(
            "{:^16}\n{:^16}\n{:<5} {:<4} {}\n{} {} {}",
            "Constants", "---------", "INDEX", "KIND", "VALUE", "-----", "----", "-----"
        )
    }

    fn local_header() -> String {
        format!(
            "{:^50}\n{:^50}\n{:<5} {:<10} {:<5} {:<5} {:<5}\n{} {} {} {} {}",
            "Locals",
            "------",
            "INDEX",
            "IDENTIFIER",
            "DEPTH",
            "KIND",
            "VALUE",
            "-----",
            "----------",
            "-----",
            "-----",
            "-----"
        )
    }
}

impl Display for DisassembledChunk<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}\n{}", self.name_header(), self.body)
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
