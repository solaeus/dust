use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Instruction, Span, Value};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    code: Vec<(u8, Span)>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn with_data(code: Vec<(u8, Span)>, constants: Vec<Value>) -> Self {
        Self { code, constants }
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.code.capacity()
    }

    pub fn read(&self, offset: usize) -> (u8, Span) {
        self.code[offset]
    }

    pub fn write(&mut self, instruction: u8, position: Span) {
        self.code.push((instruction, position));
    }

    pub fn get_constant(&self, index: usize) -> Result<&Value, ChunkError> {
        self.constants
            .get(index)
            .ok_or_else(|| ChunkError::ConstantIndexOutOfBounds(index))
    }

    pub fn push_constant(&mut self, value: Value) -> Result<u8, ChunkError> {
        let starting_length = self.constants.len();

        if starting_length + 1 > (u8::MAX as usize) {
            Err(ChunkError::Overflow)
        } else {
            self.constants.push(value);

            Ok(starting_length as u8)
        }
    }

    pub fn clear(&mut self) {
        self.code.clear();
        self.constants.clear();
    }

    pub fn disassemble(&self, name: &str) -> String {
        let mut output = String::new();

        output.push_str("== ");
        output.push_str(name);
        output.push_str(" ==\n");

        let mut next_is_index = false;

        for (offset, (byte, position)) in self.code.iter().enumerate() {
            if next_is_index {
                let index_display = format!("{position} {offset:04} INDEX {byte}\n");

                output.push_str(&index_display);

                next_is_index = false;

                continue;
            }

            let instruction = Instruction::from_byte(*byte).unwrap();
            let instruction_display =
                format!("{} {}\n", position, instruction.disassemble(self, offset));

            output.push_str(&instruction_display);

            if let Instruction::Constant = instruction {
                next_is_index = true;
            }
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
        write!(f, "{}", self.disassemble("Chunk"))
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChunkError {
    ConstantIndexOutOfBounds(usize),
    Overflow,
}
