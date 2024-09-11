use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{AnnotatedError, Identifier, Instruction, Span, Value};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    code: Vec<(u8, Span)>,
    constants: Vec<Value>,
    identifiers: Vec<Local>,
    scope_depth: usize,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            identifiers: Vec::new(),
            scope_depth: 0,
        }
    }

    pub fn with_data(
        code: Vec<(u8, Span)>,
        constants: Vec<Value>,
        identifiers: Vec<Local>,
    ) -> Self {
        Self {
            code,
            constants,
            identifiers,
            scope_depth: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }

    pub fn scope_depth(&self) -> usize {
        self.scope_depth
    }

    pub fn get_code(&self, offset: usize, position: Span) -> Result<&(u8, Span), ChunkError> {
        self.code
            .get(offset)
            .ok_or(ChunkError::CodeIndexOfBounds { offset, position })
    }

    pub fn push_code<T: Into<u8>>(&mut self, into_byte: T, position: Span) {
        self.code.push((into_byte.into(), position));
    }

    pub fn get_constant(&self, index: u8, position: Span) -> Result<&Value, ChunkError> {
        self.constants
            .get(index as usize)
            .ok_or(ChunkError::ConstantIndexOutOfBounds { index, position })
    }

    pub fn remove_constant(&mut self, index: u8, position: Span) -> Result<Value, ChunkError> {
        let index = index as usize;

        if index >= self.constants.len() {
            Err(ChunkError::ConstantIndexOutOfBounds {
                index: index as u8,
                position,
            })
        } else {
            Ok(self.constants.remove(index))
        }
    }

    pub fn push_constant(&mut self, value: Value, position: Span) -> Result<u8, ChunkError> {
        let starting_length = self.constants.len();

        if starting_length + 1 > (u8::MAX as usize) {
            Err(ChunkError::ConstantOverflow { position })
        } else {
            self.constants.push(value);

            Ok(starting_length as u8)
        }
    }

    pub fn contains_identifier(&self, identifier: &Identifier) -> bool {
        self.identifiers
            .iter()
            .any(|local| &local.identifier == identifier)
    }

    pub fn get_local(&self, index: u8, position: Span) -> Result<&Local, ChunkError> {
        self.identifiers
            .get(index as usize)
            .ok_or(ChunkError::IdentifierIndexOutOfBounds { index, position })
    }

    pub fn get_identifier(&self, index: u8, position: Span) -> Result<&Identifier, ChunkError> {
        self.identifiers
            .get(index as usize)
            .map(|local| &local.identifier)
            .ok_or(ChunkError::IdentifierIndexOutOfBounds { index, position })
    }

    pub fn get_identifier_index(
        &self,
        identifier: &Identifier,
        position: Span,
    ) -> Result<u8, ChunkError> {
        self.identifiers
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

    pub fn declare_variable(
        &mut self,
        identifier: Identifier,
        position: Span,
    ) -> Result<u8, ChunkError> {
        let starting_length = self.identifiers.len();

        if starting_length + 1 > (u8::MAX as usize) {
            Err(ChunkError::IdentifierOverflow { position })
        } else {
            self.identifiers.push(Local {
                identifier,
                depth: self.scope_depth,
            });

            Ok(starting_length as u8)
        }
    }

    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    pub fn end_scope(&mut self) {
        self.scope_depth -= 1;
    }

    pub fn clear(&mut self) {
        self.code.clear();
        self.constants.clear();
        self.identifiers.clear();
    }

    pub fn identifiers(&self) -> &[Local] {
        &self.identifiers
    }

    pub fn pop_identifier(&mut self) -> Option<Local> {
        self.identifiers.pop()
    }

    pub fn disassemble(&self, name: &str) -> String {
        let mut output = String::new();

        let name_length = name.len();
        let buffer_length = 32_usize.saturating_sub(name_length + 2);
        let name_buffer = " ".repeat(buffer_length / 2);
        let name_line = format!("{name_buffer} {name} {name_buffer}\n");

        output.push_str(&name_line);
        output.push_str("\n              Code              \n");
        output.push_str("------ ------------ ------------\n");
        output.push_str("OFFSET POSITION     INSTRUCTION\n");
        output.push_str("------ ------------ ------------\n");

        let mut previous = None;

        for (offset, (byte, position)) in self.code.iter().enumerate() {
            if let Some(
                Instruction::Constant
                | Instruction::DeclareVariable
                | Instruction::GetVariable
                | Instruction::SetVariable,
            ) = previous
            {
                previous = None;

                continue;
            }

            let instruction = Instruction::from_byte(*byte).unwrap();
            let display = format!(
                "{offset:4}   {:12} {}\n",
                position.to_string(),
                instruction.disassemble(self, offset)
            );

            output.push_str(&display);

            previous = Some(instruction);
        }

        output.push_str("\n            Constants           \n");
        output.push_str("----- ---- -----\n");
        output.push_str("INDEX KIND VALUE\n");
        output.push_str("----- ---- -----\n");

        for (index, value) in self.constants.iter().enumerate() {
            let value_kind_display = match value {
                Value::Raw(_) => "RAW ",
                Value::Reference(_) => "REF ",
                Value::Mutable(_) => "MUT ",
            };
            let display = format!("{index:3}   {value_kind_display} {value}\n");

            output.push_str(&display);
        }

        output.push_str("\n    Identifiers     \n");
        output.push_str("----- ---------- -----\n");
        output.push_str("INDEX IDENTIFIER DEPTH\n");
        output.push_str("----- ---------- -----\n");

        for (index, Local { identifier, depth }) in self.identifiers.iter().enumerate() {
            let display = format!("{index:3}   {:10} {depth}\n", identifier.as_str());
            output.push_str(&display);
        }

        output
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.disassemble("Chunk Disassembly"))
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.disassemble("Chunk Disassembly"))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Local {
    pub identifier: Identifier,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChunkError {
    CodeIndexOfBounds {
        offset: usize,
        position: Span,
    },
    ConstantOverflow {
        position: Span,
    },
    ConstantIndexOutOfBounds {
        index: u8,
        position: Span,
    },
    IdentifierIndexOutOfBounds {
        index: u8,
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
            ChunkError::ConstantOverflow { .. } => "Constant overflow",
            ChunkError::ConstantIndexOutOfBounds { .. } => "Constant index out of bounds",
            ChunkError::IdentifierIndexOutOfBounds { .. } => "Identifier index out of bounds",
            ChunkError::IdentifierOverflow { .. } => "Identifier overflow",
            ChunkError::IdentifierNotFound { .. } => "Identifier not found",
        }
    }

    fn details(&self) -> Option<String> {
        match self {
            ChunkError::CodeIndexOfBounds { offset, .. } => Some(format!("Code index: {}", offset)),
            ChunkError::ConstantIndexOutOfBounds { index, .. } => {
                Some(format!("Constant index: {}", index))
            }
            ChunkError::IdentifierIndexOutOfBounds { index, .. } => {
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
            ChunkError::ConstantIndexOutOfBounds { position, .. } => *position,
            ChunkError::IdentifierNotFound { position, .. } => *position,
            _ => todo!(),
        }
    }
}
