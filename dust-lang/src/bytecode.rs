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
            let instruction = self.read_instruction();

            match instruction {
                Instruction::Constant(index) => {
                    let value = self.read_constant(*index);

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

            self.ip += 1;
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

    pub fn read_instruction(&self) -> &Instruction {
        let (instruction, _) = &self.chunk.code[self.ip];

        instruction
    }

    pub fn read_constant(&self, index: usize) -> Value {
        self.chunk.constants[index].clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VmError {
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
    Constant(usize),
    Return,

    // Unary
    Negate,

    // Binary
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl Instruction {
    pub fn disassemble(&self, chunk: &Chunk, offset: usize) -> String {
        match self {
            Instruction::Constant(index) => {
                let value = &chunk.constants[*index];

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
    code: Vec<(Instruction, Span)>,
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

    pub fn write(&mut self, instruction: Instruction, position: Span) {
        self.code.push((instruction, position));
    }

    pub fn push_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);

        self.constants.len() - 1
    }

    pub fn clear(&mut self) {
        self.code.clear();
        self.constants.clear();
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        for (offset, (instruction, position)) in self.code.iter().enumerate() {
            println!("{} {}", position, instruction.disassemble(self, offset));
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn negation() {
        let mut chunk = Chunk::new();
        let constant = chunk.push_constant(Value::integer(42));

        chunk.write(Instruction::Constant(constant), Span(0, 1));
        chunk.write(Instruction::Negate, Span(4, 5));
        chunk.write(Instruction::Return, Span(2, 3));

        let mut vm = Vm::new(chunk);
        let result = vm.interpret();

        assert_eq!(result, Ok(Some(Value::integer(-42))));
    }

    #[test]
    fn addition() {
        let mut chunk = Chunk::new();
        let left = chunk.push_constant(Value::integer(42));
        let right = chunk.push_constant(Value::integer(23));

        chunk.write(Instruction::Constant(left), Span(0, 1));
        chunk.write(Instruction::Constant(right), Span(2, 3));
        chunk.write(Instruction::Add, Span(4, 5));
        chunk.write(Instruction::Return, Span(6, 7));

        let mut vm = Vm::new(chunk);
        let result = vm.interpret();

        assert_eq!(result, Ok(Some(Value::integer(65))));
    }

    #[test]
    fn subtraction() {
        let mut chunk = Chunk::new();
        let left = chunk.push_constant(Value::integer(42));
        let right = chunk.push_constant(Value::integer(23));

        chunk.write(Instruction::Constant(left), Span(0, 1));
        chunk.write(Instruction::Constant(right), Span(2, 3));
        chunk.write(Instruction::Subtract, Span(4, 5));
        chunk.write(Instruction::Return, Span(6, 7));

        let mut vm = Vm::new(chunk);
        let result = vm.interpret();

        assert_eq!(result, Ok(Some(Value::integer(19))));
    }

    #[test]
    fn multiplication() {
        let mut chunk = Chunk::new();
        let left = chunk.push_constant(Value::integer(42));
        let right = chunk.push_constant(Value::integer(23));

        chunk.write(Instruction::Constant(left), Span(0, 1));
        chunk.write(Instruction::Constant(right), Span(2, 3));
        chunk.write(Instruction::Multiply, Span(4, 5));
        chunk.write(Instruction::Return, Span(6, 7));

        let mut vm = Vm::new(chunk);
        let result = vm.interpret();

        assert_eq!(result, Ok(Some(Value::integer(966))));
    }

    #[test]

    fn division() {
        let mut chunk = Chunk::new();
        let left = chunk.push_constant(Value::integer(42));
        let right = chunk.push_constant(Value::integer(23));

        chunk.write(Instruction::Constant(left), Span(0, 1));
        chunk.write(Instruction::Constant(right), Span(2, 3));
        chunk.write(Instruction::Divide, Span(4, 5));
        chunk.write(Instruction::Return, Span(6, 7));

        let mut vm = Vm::new(chunk);
        let result = vm.interpret();

        assert_eq!(result, Ok(Some(Value::integer(1))));
    }
}
