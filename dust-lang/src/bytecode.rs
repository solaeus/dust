use serde::{Deserialize, Serialize};

use crate::{Span, Value, ValueError};

const STACK_SIZE: usize = 256;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

impl Vm {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::with_capacity(STACK_SIZE),
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
                    let value = self.read_constant(index as usize);

                    self.stack.push(value.clone());
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
        if self.stack.len() == STACK_SIZE {
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

        self.chunk.code[self.ip - 1]
    }

    pub fn read_constant(&self, index: usize) -> Value {
        self.chunk.constants[index].clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VmError {
    ChunkOverflow,
    InvalidInstruction(u8, Span),
    StackUnderflow,
    StackOverflow,
    Value(ValueError),
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
                let index = chunk.code[offset + 1].0 as usize;
                let value = &chunk.constants[index];

                format!("{:04} CONSTANT {} {}", offset, index, value)
            }
            Instruction::Return => format!("{:04} RETURN", offset),

            // Unary
            Instruction::Negate => format!("{:04} NEGATE", offset),

            // Binary
            Instruction::Add => format!("{:04} ADD", offset),
            Instruction::Subtract => format!("{:04} SUBTRACT", offset),
            Instruction::Multiply => format!("{:04} MULTIPLY", offset),
            Instruction::Divide => format!("{:04} DIVIDE", offset),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
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

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.code.capacity()
    }

    pub fn write(&mut self, instruction: u8, position: Span) {
        self.code.push((instruction, position));
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

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        for (offset, (byte, position)) in self.code.iter().enumerate() {
            let instruction = Instruction::from_byte(*byte).unwrap();

            println!("{} {}", position, instruction.disassemble(self, offset));
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChunkError {
    Overflow,
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
