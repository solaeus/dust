use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Chunk, ChunkError, Identifier, Span, Value, ValueError};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    globals: HashMap<Identifier, Value>,
}

impl Vm {
    const STACK_SIZE: usize = 256;

    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::with_capacity(Self::STACK_SIZE),
            globals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self) -> Result<Option<Value>, VmError> {
        while let Ok((byte, position)) = self.read().copied() {
            let instruction = Instruction::from_byte(byte)
                .ok_or_else(|| VmError::InvalidInstruction(byte, position))?;

            match instruction {
                Instruction::Constant => {
                    let (index, position) = self.read().copied()?;
                    let value = self.read_constant(index as usize)?;

                    self.stack.push(value);
                }
                Instruction::Return => {
                    let value = self.pop()?;

                    return Ok(Some(value));
                }
                Instruction::Pop => {
                    self.pop()?;
                }
                Instruction::DefineGlobal => {
                    let (index, _) = self.read().copied()?;
                    let identifier = self.chunk.get_identifier(index as usize)?.clone();
                    let value = self.pop()?;

                    self.globals.insert(identifier, value);
                }

                // Unary
                Instruction::Negate => {
                    let negated = self.pop()?.negate()?;

                    self.stack.push(negated);
                }
                Instruction::Not => {
                    let not = self.pop()?.not()?;

                    self.stack.push(not);
                }

                // Binary
                Instruction::Add => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let sum = left.add(&right)?;

                    self.stack.push(sum);
                }
                Instruction::Subtract => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let difference = left.subtract(&right)?;

                    self.stack.push(difference);
                }
                Instruction::Multiply => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let product = left.multiply(&right)?;

                    self.stack.push(product);
                }
                Instruction::Divide => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let quotient = left.divide(&right)?;

                    self.stack.push(quotient);
                }
                Instruction::Greater => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let greater = left.greater_than(&right)?;

                    self.stack.push(greater);
                }
                Instruction::Less => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let less = left.less_than(&right)?;

                    self.stack.push(less);
                }
                Instruction::GreaterEqual => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let greater_equal = left.greater_than_or_equal(&right)?;

                    self.stack.push(greater_equal);
                }
                Instruction::LessEqual => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let less_equal = left.less_than_or_equal(&right)?;

                    self.stack.push(less_equal);
                }
                Instruction::Equal => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let equal = left.equal(&right)?;

                    self.stack.push(equal);
                }
                Instruction::NotEqual => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let not_equal = left.not_equal(&right)?;

                    self.stack.push(not_equal);
                }
                Instruction::And => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let and = left.and(&right)?;

                    self.stack.push(and);
                }
                Instruction::Or => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let or = left.or(&right)?;

                    self.stack.push(or);
                }
            }
        }

        Ok(self.stack.pop())
    }

    fn push(&mut self, value: Value) -> Result<(), VmError> {
        if self.stack.len() == Self::STACK_SIZE {
            Err(VmError::StackOverflow)
        } else {
            self.stack.push(value);

            Ok(())
        }
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        if let Some(value) = self.stack.pop() {
            Ok(value)
        } else {
            Err(VmError::StackUnderflow)
        }
    }

    fn read(&mut self) -> Result<&(u8, Span), VmError> {
        let current = self.chunk.read(self.ip)?;

        self.ip += 1;

        Ok(current)
    }

    fn read_constant(&self, index: usize) -> Result<Value, VmError> {
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
    Pop = 2,
    DefineGlobal = 3,

    // Unary
    Negate = 4,
    Not = 5,

    // Binary
    Add = 6,
    Subtract = 7,
    Multiply = 8,
    Divide = 9,
    Greater = 10,
    Less = 11,
    GreaterEqual = 12,
    LessEqual = 13,
    Equal = 14,
    NotEqual = 15,
    And = 16,
    Or = 17,
}

impl Instruction {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Instruction::Constant),
            1 => Some(Instruction::Return),
            2 => Some(Instruction::Pop),
            3 => Some(Instruction::DefineGlobal),
            4 => Some(Instruction::Negate),
            5 => Some(Instruction::Not),
            6 => Some(Instruction::Add),
            7 => Some(Instruction::Subtract),
            8 => Some(Instruction::Multiply),
            9 => Some(Instruction::Divide),
            10 => Some(Instruction::Greater),
            11 => Some(Instruction::Less),
            12 => Some(Instruction::GreaterEqual),
            13 => Some(Instruction::LessEqual),
            14 => Some(Instruction::Equal),
            15 => Some(Instruction::NotEqual),
            16 => Some(Instruction::And),
            17 => Some(Instruction::Or),
            _ => None,
        }
    }

    pub fn disassemble(&self, chunk: &Chunk, offset: usize) -> String {
        match self {
            Instruction::Constant => {
                let (index, _) = chunk.read(offset + 1).unwrap();
                let value_display = chunk
                    .get_constant(*index as usize)
                    .map(|value| value.to_string())
                    .unwrap_or_else(|error| format!("{:?}", error));

                format!("{offset:04} CONSTANT {index} {value_display}")
            }
            Instruction::Return => format!("{offset:04} RETURN"),
            Instruction::Pop => format!("{offset:04} POP"),
            Instruction::DefineGlobal => {
                let (index, _) = chunk.read(offset + 1).unwrap();
                let identifier = chunk.get_identifier(*index as usize).unwrap();

                format!("{offset:04} DEFINE_GLOBAL {identifier}")
            }

            // Unary
            Instruction::Negate => format!("{offset:04} NEGATE"),
            Instruction::Not => format!("{offset:04} NOT"),

            // Binary
            Instruction::Add => format!("{offset:04} ADD"),
            Instruction::Subtract => format!("{offset:04} SUBTRACT"),
            Instruction::Multiply => format!("{offset:04} MULTIPLY"),
            Instruction::Divide => format!("{offset:04} DIVIDE"),
            Instruction::Greater => format!("{offset:04} GREATER"),
            Instruction::Less => format!("{offset:04} LESS"),
            Instruction::GreaterEqual => format!("{offset:04} GREATER_EQUAL"),
            Instruction::LessEqual => format!("{offset:04} LESS_EQUAL"),
            Instruction::Equal => format!("{offset:04} EQUAL"),
            Instruction::NotEqual => format!("{offset:04} NOT_EQUAL"),
            Instruction::And => format!("{offset:04} AND"),
            Instruction::Or => format!("{offset:04} OR"),
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
