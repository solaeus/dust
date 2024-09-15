use std::fmt::{self, Display, Formatter};

use crate::{Chunk, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction(u32);

impl Instruction {
    pub fn r#move(to_register: u8, from_register: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Move as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(from_register);

        instruction
    }

    pub fn close(to_register: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Close as u32);

        instruction.set_destination(to_register);

        instruction
    }

    pub fn load_constant(to_register: u8, constant_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::LoadConstant as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(constant_index);

        instruction
    }

    pub fn declare_local(to_register: u8, variable_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::DeclareLocal as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(variable_index);

        instruction
    }

    pub fn get_local(to_register: u8, variable_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::GetLocal as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(variable_index);

        instruction
    }

    pub fn set_local(from_register: u8, variable_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::SetLocal as u32);

        instruction.set_destination(from_register);
        instruction.set_first_argument(variable_index);

        instruction
    }

    pub fn add(to_register: u8, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Add as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(left_index);
        instruction.set_second_argument(right_index);

        instruction
    }

    pub fn subtract(to_register: u8, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Subtract as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(left_index);
        instruction.set_second_argument(right_index);

        instruction
    }

    pub fn multiply(to_register: u8, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Multiply as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(left_index);
        instruction.set_second_argument(right_index);

        instruction
    }

    pub fn divide(to_register: u8, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Divide as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(left_index);
        instruction.set_second_argument(right_index);

        instruction
    }

    pub fn negate(to_register: u8, from_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Negate as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(from_index);

        instruction
    }

    pub fn r#return() -> Instruction {
        Instruction(Operation::Return as u32)
    }

    pub fn set_first_argument_to_constant(&mut self) {
        self.0 |= 0b1000_0000;
    }

    pub fn first_argument_is_constant(&self) -> bool {
        self.0 & 0b1000_0000 != 0
    }

    pub fn set_second_argument_to_constant(&mut self) {
        self.0 |= 0b0100_0000;
    }

    pub fn second_argument_is_constant(&self) -> bool {
        self.0 & 0b0100_0000 != 0
    }

    pub fn destination(&self) -> u8 {
        (self.0 >> 24) as u8
    }

    pub fn set_destination(&mut self, destination: u8) {
        self.0 |= (destination as u32) << 24;
    }

    pub fn first_argument(&self) -> u8 {
        (self.0 >> 16) as u8
    }

    pub fn set_first_argument(&mut self, argument: u8) {
        self.0 |= (argument as u32) << 16;
    }

    pub fn second_argument(&self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub fn set_second_argument(&mut self, argument: u8) {
        self.0 |= (argument as u32) << 8;
    }

    pub fn operation(&self) -> Operation {
        Operation::from((self.0 & 0b0000_0000_0011_1111) as u8)
    }

    pub fn set_operation(&mut self, operation: Operation) {
        self.0 |= u8::from(operation) as u32;
    }

    pub fn disassemble(&self, chunk: &Chunk) -> String {
        let mut disassembled = format!("{:16} ", self.operation().to_string());

        if let Some(info) = self.disassembly_info(Some(chunk)) {
            disassembled.push_str(&info);
        }

        disassembled
    }

    pub fn disassembly_info(&self, chunk: Option<&Chunk>) -> Option<String> {
        let info = match self.operation() {
            Operation::Move => {
                format!("R({}) = R({})", self.destination(), self.first_argument())
            }
            Operation::Close => format!("R({})", self.destination()),
            Operation::LoadConstant => {
                let constant_index = self.first_argument();

                if let Some(chunk) = chunk {
                    match chunk.get_constant(constant_index, Span(0, 0)) {
                        Ok(value) => {
                            format!(
                                "R({}) = C({}) {}",
                                self.destination(),
                                constant_index,
                                value
                            )
                        }
                        Err(error) => format!(
                            "R({}) = C({}) {:?}",
                            self.destination(),
                            constant_index,
                            error
                        ),
                    }
                } else {
                    format!("R({}) = C({})", self.destination(), constant_index)
                }
            }
            Operation::DeclareLocal => {
                let local_index = self.first_argument();
                let identifier_display = if let Some(chunk) = chunk {
                    match chunk.get_identifier(local_index) {
                        Some(identifier) => identifier.to_string(),
                        None => "???".to_string(),
                    }
                } else {
                    "???".to_string()
                };

                format!(
                    "L({}) = R({}) {}",
                    local_index,
                    self.destination(),
                    identifier_display
                )
            }
            Operation::GetLocal => {
                let local_index = self.first_argument();

                format!("R({}) = L({})", self.destination(), local_index)
            }
            Operation::SetLocal => {
                let local_index = self.first_argument();
                let identifier_display = if let Some(chunk) = chunk {
                    match chunk.get_identifier(local_index) {
                        Some(identifier) => identifier.to_string(),
                        None => "???".to_string(),
                    }
                } else {
                    "???".to_string()
                };

                format!(
                    "L({}) = R({}) {}",
                    local_index,
                    self.destination(),
                    identifier_display
                )
            }
            Operation::Add => {
                let destination = self.destination();
                let first_argument = if self.first_argument_is_constant() {
                    format!("C({})", self.first_argument())
                } else {
                    format!("R({})", self.first_argument())
                };
                let second_argument = if self.second_argument_is_constant() {
                    format!("C({})", self.second_argument())
                } else {
                    format!("R({})", self.second_argument())
                };

                format!("R({destination}) = {first_argument} + {second_argument}",)
            }
            Operation::Subtract => {
                let destination = self.destination();
                let first_argument = if self.first_argument_is_constant() {
                    format!("C({})", self.first_argument())
                } else {
                    format!("R({})", self.first_argument())
                };
                let second_argument = if self.second_argument_is_constant() {
                    format!("C({})", self.second_argument())
                } else {
                    format!("R({})", self.second_argument())
                };

                format!("R({destination}) = {first_argument} - {second_argument}",)
            }
            Operation::Multiply => {
                let destination = self.destination();
                let first_argument = if self.first_argument_is_constant() {
                    format!("C({})", self.first_argument())
                } else {
                    format!("R({})", self.first_argument())
                };
                let second_argument = if self.second_argument_is_constant() {
                    format!("C({})", self.second_argument())
                } else {
                    format!("R({})", self.second_argument())
                };

                format!("R({destination}) = {first_argument} * {second_argument}",)
            }
            Operation::Divide => {
                let destination = self.destination();
                let first_argument = if self.first_argument_is_constant() {
                    format!("C({})", self.first_argument())
                } else {
                    format!("R({})", self.first_argument())
                };
                let second_argument = if self.second_argument_is_constant() {
                    format!("C({})", self.second_argument())
                } else {
                    format!("R({})", self.second_argument())
                };

                format!("R({destination}) = {first_argument} / {second_argument}",)
            }
            Operation::Negate => {
                format!("R({}) = -RC({})", self.destination(), self.first_argument())
            }
            Operation::Return => return None,
        };

        Some(info)
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(info) = self.disassembly_info(None) {
            write!(f, "{} {}", self.operation(), info)
        } else {
            write!(f, "{}", self.operation())
        }
    }
}

const MOVE: u8 = 0b0000_0000;
const CLOSE: u8 = 0b000_0001;
const LOAD_CONSTANT: u8 = 0b0000_0010;
const DECLARE_LOCAL: u8 = 0b0000_0011;
const GET_LOCAL: u8 = 0b0000_0100;
const SET_LOCAL: u8 = 0b0000_0101;
const ADD: u8 = 0b0000_0110;
const SUBTRACT: u8 = 0b0000_0111;
const MULTIPLY: u8 = 0b0000_1000;
const DIVIDE: u8 = 0b0000_1001;
const NEGATE: u8 = 0b0000_1010;
const RETURN: u8 = 0b0000_1011;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operation {
    // Stack manipulation
    Move = MOVE as isize,
    Close = CLOSE as isize,

    // Constants
    LoadConstant = LOAD_CONSTANT as isize,

    // Variables
    DeclareLocal = DECLARE_LOCAL as isize,
    GetLocal = GET_LOCAL as isize,
    SetLocal = SET_LOCAL as isize,

    // Binary operations
    Add = ADD as isize,
    Subtract = SUBTRACT as isize,
    Multiply = MULTIPLY as isize,
    Divide = DIVIDE as isize,

    // Unary operations
    Negate = NEGATE as isize,

    // Control flow
    Return = RETURN as isize,
}

impl Operation {
    pub fn is_binary(&self) -> bool {
        matches!(
            self,
            Operation::Add | Operation::Subtract | Operation::Multiply | Operation::Divide
        )
    }
}

impl From<u8> for Operation {
    fn from(byte: u8) -> Self {
        match byte {
            MOVE => Operation::Move,
            CLOSE => Operation::Close,
            LOAD_CONSTANT => Operation::LoadConstant,
            DECLARE_LOCAL => Operation::DeclareLocal,
            GET_LOCAL => Operation::GetLocal,
            SET_LOCAL => Operation::SetLocal,
            ADD => Operation::Add,
            SUBTRACT => Operation::Subtract,
            MULTIPLY => Operation::Multiply,
            DIVIDE => Operation::Divide,
            NEGATE => Operation::Negate,
            _ => Operation::Return,
        }
    }
}

impl From<Operation> for u8 {
    fn from(operation: Operation) -> Self {
        match operation {
            Operation::Move => MOVE,
            Operation::Close => CLOSE,
            Operation::LoadConstant => LOAD_CONSTANT,
            Operation::DeclareLocal => DECLARE_LOCAL,
            Operation::GetLocal => GET_LOCAL,
            Operation::SetLocal => SET_LOCAL,
            Operation::Add => ADD,
            Operation::Subtract => SUBTRACT,
            Operation::Multiply => MULTIPLY,
            Operation::Divide => DIVIDE,
            Operation::Negate => NEGATE,
            Operation::Return => RETURN,
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Operation::Move => write!(f, "MOVE"),
            Operation::Close => write!(f, "CLOSE"),
            Operation::LoadConstant => write!(f, "LOAD_CONSTANT"),
            Operation::DeclareLocal => write!(f, "DECLARE_LOCAL"),
            Operation::GetLocal => write!(f, "GET_LOCAL"),
            Operation::SetLocal => write!(f, "SET_LOCAL"),
            Operation::Add => write!(f, "ADD"),
            Operation::Subtract => write!(f, "SUBTRACT"),
            Operation::Multiply => write!(f, "MULTIPLY"),
            Operation::Divide => write!(f, "DIVIDE"),
            Operation::Negate => write!(f, "NEGATE"),
            Operation::Return => write!(f, "RETURN"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r#move() {
        let mut instruction = Instruction::r#move(255, 255);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Move);
        assert_eq!(instruction.destination(), 255);
        assert_eq!(instruction.first_argument(), 255);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn close() {
        let mut instruction = Instruction::close(255);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Close);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn load_constant() {
        let mut instruction = Instruction::load_constant(255, 255);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::LoadConstant);
        assert_eq!(instruction.destination(), 255);
        assert_eq!(instruction.first_argument(), 255);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn declare_local() {
        let mut instruction = Instruction::declare_local(255, 255);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::DeclareLocal);
        assert_eq!(instruction.destination(), 255);
        assert_eq!(instruction.first_argument(), 255);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn add() {
        let mut instruction = Instruction::add(255, 255, 2);

        instruction.set_operation(Operation::Add);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Add);
        assert_eq!(instruction.destination(), 255);
        assert_eq!(instruction.first_argument(), 255);
        assert_eq!(instruction.second_argument(), 2);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn subtract() {
        let mut instruction = Instruction::subtract(255, 255, 255);

        instruction.set_operation(Operation::Subtract);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Subtract);
        assert_eq!(instruction.destination(), 255);
        assert_eq!(instruction.first_argument(), 255);
        assert_eq!(instruction.second_argument(), 255);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn multiply() {
        let mut instruction = Instruction::multiply(255, 255, 255);

        instruction.set_operation(Operation::Multiply);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Multiply);
        assert_eq!(instruction.destination(), 255);
        assert_eq!(instruction.first_argument(), 255);
        assert_eq!(instruction.second_argument(), 255);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn divide() {
        let mut instruction = Instruction::divide(255, 255, 255);

        instruction.set_operation(Operation::Divide);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Divide);
        assert_eq!(instruction.destination(), 255);
        assert_eq!(instruction.first_argument(), 255);
        assert_eq!(instruction.second_argument(), 255);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn negate() {
        let mut instruction = Instruction::negate(255, 255);

        instruction.set_operation(Operation::Negate);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Negate);
        assert_eq!(instruction.destination(), 255);
        assert_eq!(instruction.first_argument(), 255);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn r#return() {
        let mut instruction = Instruction::r#return();

        instruction.set_operation(Operation::Return);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Return);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }
}
