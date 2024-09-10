use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{identifier_stack::Local, Identifier, IdentifierStack, Instruction, Span, Value};

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

    pub fn push_identifier(&mut self, identifier: Identifier) -> Result<u8, ChunkError> {
        let starting_length = self.identifiers.local_count();

        if starting_length + 1 > (u8::MAX as usize) {
            Err(ChunkError::IdentifierOverflow)
        } else {
            self.identifiers.declare(identifier);

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

        output.push_str("== ");
        output.push_str(name);
        output.push_str(" ==\n--Code--\n");
        output.push_str("OFFSET INSTRUCTION          POSITION\n");

        let mut previous = None;

        for (offset, (byte, position)) in self.code.iter().enumerate() {
            if let Some(
                Instruction::Constant
                | Instruction::DefineVariable
                | Instruction::GetVariable
                | Instruction::SetVariable,
            ) = previous
            {
                previous = None;

                continue;
            }

            let instruction = Instruction::from_byte(*byte).unwrap();
            let display = format!("{offset:04}   {}", instruction.disassemble(self, offset));
            let display_with_postion = format!("{display:27} {position}\n");

            previous = Some(instruction);

            output.push_str(&display_with_postion);
        }

        output.push_str("--Constants--\n");
        output.push_str("INDEX KIND VALUE\n");

        for (index, value) in self.constants.iter().enumerate() {
            let value_kind_display = match value {
                Value::Raw(_) => "RAW ",
                Value::Reference(_) => "REF ",
                Value::Mutable(_) => "MUT ",
            };
            let display = format!("{index:04}  {value_kind_display} {value}\n");

            output.push_str(&display);
        }

        output.push_str("--Identifiers--\n");
        output.push_str("INDEX IDENTIFIER DEPTH\n");

        for (index, Local { identifier, depth }) in self.identifiers.iter().enumerate() {
            let display = format!("{index:04}  {:10} {depth}\n", identifier.as_str());
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

impl ChunkError {
    pub fn title(&self) -> &'static str {
        "Chunk Error"
    }

    pub fn description(&self) -> String {
        match self {
            Self::CodeIndexOfBounds(offset) => format!("{offset} is out of bounds",),
            Self::ConstantOverflow => "More than 256 constants declared in one chunk".to_string(),
            Self::ConstantIndexOutOfBounds(index) => {
                format!("{index} is out of bounds")
            }
            Self::IdentifierIndexOutOfBounds(index) => {
                format!("{index} is out of bounds")
            }
            Self::IdentifierOverflow => {
                "More than 256 identifiers declared in one chunk".to_string()
            }
            Self::IdentifierNotFound(identifier) => {
                format!("{} does not exist in this scope", identifier)
            }
        }
    }
}
