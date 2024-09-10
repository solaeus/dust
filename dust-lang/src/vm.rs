use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Chunk, ChunkError, Identifier, Span, Value, ValueError};

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

    pub fn run(&mut self) -> Result<Option<Value>, VmError> {
        while let Ok((byte, position)) = self.read().copied() {
            let instruction = Instruction::from_byte(byte)
                .ok_or_else(|| VmError::InvalidInstruction(byte, position))?;

            log::trace!("Running instruction {instruction} at {position}");

            match instruction {
                Instruction::Constant => {
                    let (index, _) = self.read().copied()?;
                    let value = self.read_constant(index)?;

                    self.push(value)?;
                }
                Instruction::Return => {
                    let value = self.pop()?;

                    return Ok(Some(value));
                }
                Instruction::Pop => {
                    self.pop()?;
                }

                // Variables
                Instruction::DefineVariable => {
                    let (index, _) = *self.read()?;
                    let value = self.read_constant(index)?;

                    self.stack.insert(index as usize, value);
                }
                Instruction::GetVariable => {
                    let (index, _) = *self.read()?;
                    let value = self.stack[index as usize].clone();

                    self.push(value)?;
                }
                Instruction::SetVariable => {
                    let (index, _) = *self.read()?;
                    let identifier = self.chunk.get_identifier(index)?.clone();

                    if !self.chunk.contains_identifier(&identifier) {
                        return Err(VmError::UndefinedVariable(identifier, position));
                    }

                    let value = self.pop()?;

                    self.stack[index as usize] = value;
                }

                // Unary
                Instruction::Negate => {
                    let negated = self.pop()?.negate()?;

                    self.push(negated)?;
                }
                Instruction::Not => {
                    let not = self.pop()?.not()?;

                    self.push(not)?;
                }

                // Binary
                Instruction::Add => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let sum = left.add(&right)?;

                    self.push(sum)?;
                }
                Instruction::Subtract => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let difference = left.subtract(&right)?;

                    self.push(difference)?;
                }
                Instruction::Multiply => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let product = left.multiply(&right)?;

                    self.push(product)?;
                }
                Instruction::Divide => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let quotient = left.divide(&right)?;

                    self.push(quotient)?;
                }
                Instruction::Greater => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let greater = left.greater_than(&right)?;

                    self.push(greater)?;
                }
                Instruction::Less => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let less = left.less_than(&right)?;

                    self.push(less)?;
                }
                Instruction::GreaterEqual => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let greater_equal = left.greater_than_or_equal(&right)?;

                    self.push(greater_equal)?;
                }
                Instruction::LessEqual => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let less_equal = left.less_than_or_equal(&right)?;

                    self.push(less_equal)?;
                }
                Instruction::Equal => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let equal = left.equal(&right)?;

                    self.push(equal)?;
                }
                Instruction::NotEqual => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let not_equal = left.not_equal(&right)?;

                    self.push(not_equal)?;
                }
                Instruction::And => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let and = left.and(&right)?;

                    self.push(and)?;
                }
                Instruction::Or => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    let or = left.or(&right)?;

                    self.push(or)?;
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
        let current = self.chunk.get_code(self.ip)?;

        self.ip += 1;

        Ok(current)
    }

    fn read_constant(&self, index: u8) -> Result<Value, VmError> {
        Ok(self.chunk.get_constant(index)?.clone())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VmError {
    InvalidInstruction(u8, Span),
    StackUnderflow,
    StackOverflow,
    UndefinedVariable(Identifier, Span),

    // Wrappers for foreign errors
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

    // Variables
    DefineVariable = 3,
    GetVariable = 4,
    SetVariable = 5,

    // Unary
    Negate = 6,
    Not = 7,

    // Binary
    Add = 8,
    Subtract = 9,
    Multiply = 10,
    Divide = 11,
    Greater = 12,
    Less = 13,
    GreaterEqual = 14,
    LessEqual = 15,
    Equal = 16,
    NotEqual = 17,
    And = 18,
    Or = 19,
}

impl Instruction {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Instruction::Constant),
            1 => Some(Instruction::Return),
            2 => Some(Instruction::Pop),
            3 => Some(Instruction::DefineVariable),
            4 => Some(Instruction::GetVariable),
            5 => Some(Instruction::SetVariable),
            6 => Some(Instruction::Negate),
            7 => Some(Instruction::Not),
            8 => Some(Instruction::Add),
            9 => Some(Instruction::Subtract),
            10 => Some(Instruction::Multiply),
            11 => Some(Instruction::Divide),
            12 => Some(Instruction::Greater),
            13 => Some(Instruction::Less),
            14 => Some(Instruction::GreaterEqual),
            15 => Some(Instruction::LessEqual),
            16 => Some(Instruction::Equal),
            17 => Some(Instruction::NotEqual),
            18 => Some(Instruction::And),
            19 => Some(Instruction::Or),
            _ => None,
        }
    }

    pub fn disassemble(&self, chunk: &Chunk, offset: usize) -> String {
        match self {
            Instruction::Constant => {
                let (index_display, value_display) =
                    if let Ok((index, _)) = chunk.get_code(offset + 1) {
                        let index_string = index.to_string();
                        let value_string = chunk
                            .get_constant(*index)
                            .map(|value| value.to_string())
                            .unwrap_or_else(|error| format!("{:?}", error));

                        (index_string, value_string)
                    } else {
                        let index = "ERROR".to_string();
                        let value = "ERROR".to_string();

                        (index, value)
                    };

                format!("CONSTANT {index_display} {value_display}")
            }
            Instruction::Return => format!("{offset:04} RETURN"),
            Instruction::Pop => format!("{offset:04} POP"),

            // Variables
            Instruction::DefineVariable => {
                let (index, _) = chunk.get_code(offset + 1).unwrap();
                let identifier_display = match chunk.get_identifier(*index) {
                    Ok(identifier) => identifier.to_string(),
                    Err(error) => format!("{:?}", error),
                };

                format!("DEFINE_VARIABLE {identifier_display} {index}")
            }
            Instruction::GetVariable => {
                let (index, _) = chunk.get_code(offset + 1).unwrap();
                let identifier_display = match chunk.get_identifier(*index) {
                    Ok(identifier) => identifier.to_string(),
                    Err(error) => format!("{:?}", error),
                };

                format!("GET_VARIABLE {identifier_display} {index}")
            }

            Instruction::SetVariable => {
                let (index, _) = chunk.get_code(offset + 1).unwrap();
                let identifier_display = match chunk.get_identifier(*index) {
                    Ok(identifier) => identifier.to_string(),
                    Err(error) => format!("{:?}", error),
                };

                format!("SET_VARIABLE {identifier_display} {index}")
            }

            // Unary
            Instruction::Negate => "NEGATE".to_string(),
            Instruction::Not => "NOT".to_string(),

            // Binary
            Instruction::Add => "ADD".to_string(),
            Instruction::Subtract => "SUBTRACT".to_string(),
            Instruction::Multiply => "MULTIPLY".to_string(),
            Instruction::Divide => "DIVIDE".to_string(),
            Instruction::Greater => "GREATER".to_string(),
            Instruction::Less => "LESS".to_string(),
            Instruction::GreaterEqual => "GREATER_EQUAL".to_string(),
            Instruction::LessEqual => "LESS_EQUAL".to_string(),
            Instruction::Equal => "EQUAL".to_string(),
            Instruction::NotEqual => "NOT_EQUAL".to_string(),
            Instruction::And => "AND".to_string(),
            Instruction::Or => "OR".to_string(),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn negation() {
        let mut chunk = Chunk::new();
        let constant = chunk.push_constant(Value::integer(42)).unwrap();

        chunk.push_code(Instruction::Constant as u8, Span(0, 1));
        chunk.push_code(constant, Span(2, 3));
        chunk.push_code(Instruction::Negate as u8, Span(4, 5));
        chunk.push_code(Instruction::Return as u8, Span(2, 3));

        let mut vm = Vm::new(chunk);
        let result = vm.run();

        assert_eq!(result, Ok(Some(Value::integer(-42))));
    }

    #[test]
    fn addition() {
        let mut chunk = Chunk::new();
        let left = chunk.push_constant(Value::integer(42)).unwrap();
        let right = chunk.push_constant(Value::integer(23)).unwrap();

        chunk.push_code(Instruction::Constant as u8, Span(0, 1));
        chunk.push_code(left, Span(2, 3));
        chunk.push_code(Instruction::Constant as u8, Span(4, 5));
        chunk.push_code(right, Span(6, 7));
        chunk.push_code(Instruction::Add as u8, Span(8, 9));
        chunk.push_code(Instruction::Return as u8, Span(10, 11));

        let mut vm = Vm::new(chunk);
        let result = vm.run();

        assert_eq!(result, Ok(Some(Value::integer(65))));
    }

    #[test]
    fn subtraction() {
        let mut chunk = Chunk::new();
        let left = chunk.push_constant(Value::integer(42)).unwrap();
        let right = chunk.push_constant(Value::integer(23)).unwrap();

        chunk.push_code(Instruction::Constant as u8, Span(0, 1));
        chunk.push_code(left, Span(2, 3));
        chunk.push_code(Instruction::Constant as u8, Span(4, 5));
        chunk.push_code(right, Span(6, 7));
        chunk.push_code(Instruction::Subtract as u8, Span(8, 9));
        chunk.push_code(Instruction::Return as u8, Span(10, 11));

        let mut vm = Vm::new(chunk);
        let result = vm.run();

        assert_eq!(result, Ok(Some(Value::integer(19))));
    }

    #[test]
    fn multiplication() {
        let mut chunk = Chunk::new();
        let left = chunk.push_constant(Value::integer(42)).unwrap();
        let right = chunk.push_constant(Value::integer(23)).unwrap();

        chunk.push_code(Instruction::Constant as u8, Span(0, 1));
        chunk.push_code(left, Span(2, 3));
        chunk.push_code(Instruction::Constant as u8, Span(4, 5));
        chunk.push_code(right, Span(6, 7));
        chunk.push_code(Instruction::Multiply as u8, Span(8, 9));
        chunk.push_code(Instruction::Return as u8, Span(10, 11));

        let mut vm = Vm::new(chunk);
        let result = vm.run();

        assert_eq!(result, Ok(Some(Value::integer(966))));
    }

    #[test]

    fn division() {
        let mut chunk = Chunk::new();
        let left = chunk.push_constant(Value::integer(42)).unwrap();
        let right = chunk.push_constant(Value::integer(23)).unwrap();

        chunk.push_code(Instruction::Constant as u8, Span(0, 1));
        chunk.push_code(left, Span(2, 3));
        chunk.push_code(Instruction::Constant as u8, Span(4, 5));
        chunk.push_code(right, Span(6, 7));
        chunk.push_code(Instruction::Divide as u8, Span(8, 9));
        chunk.push_code(Instruction::Return as u8, Span(10, 11));

        let mut vm = Vm::new(chunk);
        let result = vm.run();

        assert_eq!(result, Ok(Some(Value::integer(1))));
    }
}
