use serde::{Deserialize, Serialize};

use crate::{Chunk, ChunkError, Span, Value, ValueError};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

impl Vm {
    const STACK_SIZE: usize = 256;

    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::with_capacity(Self::STACK_SIZE),
        }
    }

    pub fn interpret(&mut self) -> Result<Option<Value>, VmError> {
        loop {
            let (byte, position) = self.read();
            let instruction = Instruction::from_byte(byte)
                .ok_or_else(|| VmError::InvalidInstruction(byte, position))?;

            match instruction {
                Instruction::Constant => {
                    let (index, _) = self.read();
                    let value = self.read_constant(index as usize)?;

                    self.stack.push(value);
                }
                Instruction::Return => {
                    let value = self.pop()?;

                    return Ok(Some(value));
                }

                // Unary
                Instruction::Negate => {
                    let negated = self.pop()?.negate()?;

                    self.stack.push(negated);
                }

                // Binary
                Instruction::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;

                    let sum = a.add(&b)?;

                    self.stack.push(sum);
                }
                Instruction::Subtract => {
                    let b = self.pop()?;
                    let a = self.pop()?;

                    let difference = a.subtract(&b)?;

                    self.stack.push(difference);
                }
                Instruction::Multiply => {
                    let b = self.pop()?;
                    let a = self.pop()?;

                    let product = a.multiply(&b)?;

                    self.stack.push(product);
                }
                Instruction::Divide => {
                    let b = self.pop()?;
                    let a = self.pop()?;

                    let quotient = a.divide(&b)?;

                    self.stack.push(quotient);
                }
            }
        }
    }

    pub fn push(&mut self, value: Value) -> Result<(), VmError> {
        if self.stack.len() == Self::STACK_SIZE {
            Err(VmError::StackOverflow)
        } else {
            self.stack.push(value);

            Ok(())
        }
    }

    pub fn pop(&mut self) -> Result<Value, VmError> {
        if let Some(value) = self.stack.pop() {
            Ok(value)
        } else {
            Err(VmError::StackUnderflow)
        }
    }

    pub fn read(&mut self) -> (u8, Span) {
        self.ip += 1;

        self.chunk.read(self.ip - 1)
    }

    pub fn read_constant(&self, index: usize) -> Result<Value, VmError> {
        Ok(self.chunk.get_constant(index)?.clone())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VmError {
    InvalidInstruction(u8, Span),
    StackUnderflow,
    StackOverflow,

    Chunk(ChunkError),
    Value(ValueError),
}

impl From<ChunkError> for VmError {
    fn from(error: ChunkError) -> Self {
        Self::Chunk(error)
    }
}

impl From<ValueError> for VmError {
    fn from(error: ValueError) -> Self {
        Self::Value(error)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Instruction {
    Constant = 0,
    Return = 1,

    // Unary
    Negate = 2,

    // Binary
    Add = 3,
    Subtract = 4,
    Multiply = 5,
    Divide = 6,
}

impl Instruction {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::Constant),
            1 => Some(Self::Return),

            // Unary
            2 => Some(Self::Negate),

            // Binary
            3 => Some(Self::Add),
            4 => Some(Self::Subtract),
            5 => Some(Self::Multiply),
            6 => Some(Self::Divide),

            _ => None,
        }
    }

    pub fn disassemble(&self, chunk: &Chunk, offset: usize) -> String {
        match self {
            Instruction::Constant => {
                let (index, _) = chunk.read(offset + 1);
                let value_display = chunk
                    .get_constant(index as usize)
                    .map(|value| value.to_string())
                    .unwrap_or_else(|error| format!("{:?}", error));

                format!("{offset:04} CONSTANT {index} {value_display}")
            }
            Instruction::Return => format!("{offset:04} RETURN"),

            // Unary
            Instruction::Negate => format!("{offset:04} NEGATE"),

            // Binary
            Instruction::Add => format!("{offset:04} ADD"),
            Instruction::Subtract => format!("{offset:04} SUBTRACT"),
            Instruction::Multiply => format!("{offset:04} MULTIPLY"),
            Instruction::Divide => format!("{offset:04} DIVIDE"),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn negation() {
        let mut chunk = Chunk::new();
        let constant = chunk.push_constant(Value::integer(42)).unwrap();

        chunk.write(Instruction::Constant as u8, Span(0, 1));
        chunk.write(constant, Span(2, 3));
        chunk.write(Instruction::Negate as u8, Span(4, 5));
        chunk.write(Instruction::Return as u8, Span(2, 3));

        let mut vm = Vm::new(chunk);
        let result = vm.interpret();

        assert_eq!(result, Ok(Some(Value::integer(-42))));
    }

    #[test]
    fn addition() {
        let mut chunk = Chunk::new();
        let left = chunk.push_constant(Value::integer(42)).unwrap();
        let right = chunk.push_constant(Value::integer(23)).unwrap();

        chunk.write(Instruction::Constant as u8, Span(0, 1));
        chunk.write(left, Span(2, 3));
        chunk.write(Instruction::Constant as u8, Span(4, 5));
        chunk.write(right, Span(6, 7));
        chunk.write(Instruction::Add as u8, Span(8, 9));
        chunk.write(Instruction::Return as u8, Span(10, 11));

        let mut vm = Vm::new(chunk);
        let result = vm.interpret();

        assert_eq!(result, Ok(Some(Value::integer(65))));
    }

    #[test]
    fn subtraction() {
        let mut chunk = Chunk::new();
        let left = chunk.push_constant(Value::integer(42)).unwrap();
        let right = chunk.push_constant(Value::integer(23)).unwrap();

        chunk.write(Instruction::Constant as u8, Span(0, 1));
        chunk.write(left, Span(2, 3));
        chunk.write(Instruction::Constant as u8, Span(4, 5));
        chunk.write(right, Span(6, 7));
        chunk.write(Instruction::Subtract as u8, Span(8, 9));
        chunk.write(Instruction::Return as u8, Span(10, 11));

        let mut vm = Vm::new(chunk);
        let result = vm.interpret();

        assert_eq!(result, Ok(Some(Value::integer(19))));
    }

    #[test]
    fn multiplication() {
        let mut chunk = Chunk::new();
        let left = chunk.push_constant(Value::integer(42)).unwrap();
        let right = chunk.push_constant(Value::integer(23)).unwrap();

        chunk.write(Instruction::Constant as u8, Span(0, 1));
        chunk.write(left, Span(2, 3));
        chunk.write(Instruction::Constant as u8, Span(4, 5));
        chunk.write(right, Span(6, 7));
        chunk.write(Instruction::Multiply as u8, Span(8, 9));
        chunk.write(Instruction::Return as u8, Span(10, 11));

        let mut vm = Vm::new(chunk);
        let result = vm.interpret();

        assert_eq!(result, Ok(Some(Value::integer(966))));
    }

    #[test]

    fn division() {
        let mut chunk = Chunk::new();
        let left = chunk.push_constant(Value::integer(42)).unwrap();
        let right = chunk.push_constant(Value::integer(23)).unwrap();

        chunk.write(Instruction::Constant as u8, Span(0, 1));
        chunk.write(left, Span(2, 3));
        chunk.write(Instruction::Constant as u8, Span(4, 5));
        chunk.write(right, Span(6, 7));
        chunk.write(Instruction::Divide as u8, Span(8, 9));
        chunk.write(Instruction::Return as u8, Span(10, 11));

        let mut vm = Vm::new(chunk);
        let result = vm.interpret();

        assert_eq!(result, Ok(Some(Value::integer(1))));
    }
}
