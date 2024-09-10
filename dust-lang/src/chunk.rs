use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    identifier_stack::Local, Identifier, IdentifierStack, Instruction, Span, Value, ValueLocation,
};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    code: Vec<(u8, Span)>,
    constants: Vec<Value>,
    identifiers: IdentifierStack,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            identifiers: IdentifierStack::new(),
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
            identifiers: IdentifierStack::with_data(identifiers, 0),
        }
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }

    pub fn get_code(&self, offset: usize) -> Result<&(u8, Span), ChunkError> {
        self.code
            .get(offset)
            .ok_or(ChunkError::CodeIndexOfBounds(offset))
    }

    pub fn push_code(&mut self, instruction: u8, position: Span) {
        self.code.push((instruction, position));
    }

    pub fn get_constant(&self, index: u8) -> Result<&Value, ChunkError> {
        self.constants
            .get(index as usize)
            .ok_or(ChunkError::ConstantIndexOutOfBounds(index))
    }

    pub fn remove_constant(&mut self, index: u8) -> Result<Value, ChunkError> {
        let index = index as usize;

        if index >= self.constants.len() {
            Err(ChunkError::ConstantIndexOutOfBounds(index as u8))
        } else {
            Ok(self.constants.remove(index))
        }
    }

    pub fn push_constant(&mut self, value: Value) -> Result<u8, ChunkError> {
        let starting_length = self.constants.len();

        if starting_length + 1 > (u8::MAX as usize) {
            Err(ChunkError::ConstantOverflow)
        } else {
            self.constants.push(value);

            Ok(starting_length as u8)
        }
    }

    pub fn contains_identifier(&self, identifier: &Identifier) -> bool {
        self.identifiers.contains(identifier)
    }

    pub fn get_local(&self, index: u8) -> Result<&Local, ChunkError> {
        self.identifiers
            .get(index as usize)
            .ok_or(ChunkError::IdentifierIndexOutOfBounds(index))
    }

    pub fn get_identifier(&self, index: u8) -> Result<&Identifier, ChunkError> {
        self.identifiers
            .get(index as usize)
            .map(|local| &local.identifier)
            .ok_or(ChunkError::IdentifierIndexOutOfBounds(index))
    }

    pub fn get_identifier_index(&self, identifier: &Identifier) -> Result<u8, ChunkError> {
        self.identifiers
            .get_index(identifier)
            .ok_or(ChunkError::IdentifierNotFound(identifier.clone()))
    }

    pub fn push_constant_identifier(&mut self, identifier: Identifier) -> Result<u8, ChunkError> {
        let starting_length = self.identifiers.local_count();

        if starting_length + 1 > (u8::MAX as usize) {
            Err(ChunkError::IdentifierOverflow)
        } else {
            self.identifiers
                .declare(identifier, ValueLocation::ConstantStack);

            Ok(starting_length as u8)
        }
    }

    pub fn push_runtime_identifier(&mut self, identifier: Identifier) -> Result<u8, ChunkError> {
        let starting_length = self.identifiers.local_count();

        if starting_length + 1 > (u8::MAX as usize) {
            Err(ChunkError::IdentifierOverflow)
        } else {
            self.identifiers
                .declare(identifier, ValueLocation::RuntimeStack);

            Ok(starting_length as u8)
        }
    }

    pub fn clear(&mut self) {
        self.code.clear();
        self.constants.clear();
        self.identifiers.clear();
    }

    pub fn disassemble(&self, name: &str) -> String {
        let mut output = String::new();

        output.push_str("# ");
        output.push_str(name);
        output.push_str("\n\n## Code\n");
        output.push_str("------ ------------ ------------\n");
        output.push_str("OFFSET POSITION     INSTRUCTION\n");
        output.push_str("------ ------------ ------------\n");

        let mut previous = None;

        for (offset, (byte, position)) in self.code.iter().enumerate() {
            if let Some(
                Instruction::Constant
                | Instruction::DefineVariableConstant
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

            previous = Some(instruction);

            output.push_str(&display);
        }

        output.push_str("\n## Constants\n");
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

        output.push_str("\n## Identifiers\n");
        output.push_str("----- ---------- -------- -----\n");
        output.push_str("INDEX IDENTIFIER LOCATION DEPTH\n");
        output.push_str("----- ---------- -------- -----\n");

        for (
            index,
            Local {
                identifier,
                depth,
                value_location,
            },
        ) in self.identifiers.iter().enumerate()
        {
            let display = format!(
                "{index:3}   {:10} {value_location} {depth}\n",
                identifier.as_str()
            );
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

#[derive(Debug, Clone, PartialEq)]
pub enum ChunkError {
    CodeIndexOfBounds(usize),
    ConstantOverflow,
    ConstantIndexOutOfBounds(u8),
    IdentifierIndexOutOfBounds(u8),
    IdentifierOverflow,
    IdentifierNotFound(Identifier),
}

impl Display for ChunkError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ChunkError::CodeIndexOfBounds(offset) => {
                write!(f, "Code index out of bounds: {}", offset)
            }
            ChunkError::ConstantOverflow => write!(f, "Constant overflow"),
            ChunkError::ConstantIndexOutOfBounds(index) => {
                write!(f, "Constant index out of bounds: {}", index)
            }
            ChunkError::IdentifierIndexOutOfBounds(index) => {
                write!(f, "Identifier index out of bounds: {}", index)
            }
            ChunkError::IdentifierOverflow => write!(f, "Identifier overflow"),
            ChunkError::IdentifierNotFound(identifier) => {
                write!(f, "Identifier not found: {}", identifier)
            }
        }
    }
}
