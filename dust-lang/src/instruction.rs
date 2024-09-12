use std::fmt::{self, Display, Formatter};

use crate::{Chunk, Span};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Instruction {
    pub operation: Operation,
    pub to_register: u8,
    pub arguments: [u8; 2],
}

impl Instruction {
    pub fn r#move(to_register: u8, from_register: u8) -> Instruction {
        Instruction {
            operation: Operation::Move,
            to_register,
            arguments: [from_register, 0],
        }
    }

    pub fn close(to_register: u8) -> Instruction {
        Instruction {
            operation: Operation::Close,
            to_register,
            arguments: [0, 0],
        }
    }

    pub fn load_constant(to_register: u8, constant_index: u16) -> Instruction {
        Instruction {
            operation: Operation::LoadConstant,
            to_register,
            arguments: constant_index.to_le_bytes(),
        }
    }

    pub fn declare_variable(to_register: u8, variable_index: u16) -> Instruction {
        Instruction {
            operation: Operation::DeclareVariable,
            to_register,
            arguments: variable_index.to_le_bytes(),
        }
    }

    pub fn get_variable(to_register: u8, variable_index: u16) -> Instruction {
        Instruction {
            operation: Operation::GetVariable,
            to_register,
            arguments: variable_index.to_le_bytes(),
        }
    }

    pub fn set_variable(from_register: u8, variable_index: u16) -> Instruction {
        Instruction {
            operation: Operation::SetVariable,
            to_register: from_register,
            arguments: variable_index.to_le_bytes(),
        }
    }

    pub fn add(to_register: u8, left_register: u8, right_register: u8) -> Instruction {
        Instruction {
            operation: Operation::Add,
            to_register,
            arguments: [left_register, right_register],
        }
    }

    pub fn subtract(to_register: u8, left_register: u8, right_register: u8) -> Instruction {
        Instruction {
            operation: Operation::Subtract,
            to_register,
            arguments: [left_register, right_register],
        }
    }

    pub fn multiply(to_register: u8, left_register: u8, right_register: u8) -> Instruction {
        Instruction {
            operation: Operation::Multiply,
            to_register,
            arguments: [left_register, right_register],
        }
    }

    pub fn divide(to_register: u8, left_register: u8, right_register: u8) -> Instruction {
        Instruction {
            operation: Operation::Divide,
            to_register,
            arguments: [left_register, right_register],
        }
    }

    pub fn negate(to_register: u8, from_register: u8) -> Instruction {
        Instruction {
            operation: Operation::Negate,
            to_register,
            arguments: [from_register, 0],
        }
    }

    pub fn r#return() -> Instruction {
        Instruction {
            operation: Operation::Return,
            to_register: 0,
            arguments: [0, 0],
        }
    }

    pub fn disassemble(&self, chunk: &Chunk) -> String {
        match self.operation {
            Operation::LoadConstant => {
                let constant_index = u16::from_le_bytes(self.arguments);
                let constant_display = match chunk.get_constant(constant_index, Span(0, 0)) {
                    Ok(value) => value.to_string(),
                    Err(error) => format!("{:?}", error),
                };

                format!("{self} {constant_display}")
            }
            _ => format!("{self}"),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.operation {
            Operation::Move => write!(f, "MOVE R{} R{}", self.to_register, self.arguments[0]),
            Operation::Close => write!(f, "CLOSE R{}", self.to_register),
            Operation::LoadConstant => {
                let constant_index = u16::from_le_bytes(self.arguments);

                write!(f, "LOAD_CONSTANT R{} C{}", self.to_register, constant_index)
            }
            Operation::DeclareVariable => {
                let variable_index = u16::from_le_bytes([self.arguments[0], self.arguments[1]]);

                write!(
                    f,
                    "DECLARE_VARIABLE V{} R{}",
                    variable_index, self.to_register
                )
            }
            Operation::GetVariable => {
                let variable_index = u16::from_le_bytes([self.arguments[0], self.arguments[1]]);

                write!(f, "GET_VARIABLE R{} V{}", self.to_register, variable_index)
            }
            Operation::SetVariable => {
                let variable_index = u16::from_le_bytes([self.arguments[0], self.arguments[1]]);

                write!(f, "SET_VARIABLE V{} R{}", variable_index, self.to_register)
            }
            Operation::Add => {
                write!(
                    f,
                    "ADD R{} = R{} + R{}",
                    self.to_register, self.arguments[0], self.arguments[1]
                )
            }
            Operation::Subtract => {
                write!(
                    f,
                    "SUBTRACT R{} = R{} - R{}",
                    self.to_register, self.arguments[0], self.arguments[1]
                )
            }
            Operation::Multiply => {
                write!(
                    f,
                    "MULTIPLY R{} = R{} * R{}",
                    self.to_register, self.arguments[0], self.arguments[1]
                )
            }
            Operation::Divide => {
                write!(
                    f,
                    "DIVIDE R{} = R{} / R{}",
                    self.to_register, self.arguments[0], self.arguments[1]
                )
            }
            Operation::Negate => {
                write!(f, "NEGATE R{} = !R{}", self.to_register, self.arguments[0])
            }
            Operation::Return => {
                write!(f, "RETURN")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operation {
    // Stack manipulation
    Move,
    Close,

    // Constants
    LoadConstant,

    // Variables
    DeclareVariable,
    GetVariable,
    SetVariable,

    // Binary operations
    Add,
    Subtract,
    Multiply,
    Divide,

    // Unary operations
    Negate,

    // Control flow
    Return,
}

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Operation::Move => write!(f, "MOVE"),
            Operation::Close => write!(f, "CLOSE"),
            Operation::LoadConstant => write!(f, "LOAD_CONSTANT"),
            Operation::DeclareVariable => write!(f, "DECLARE_VARIABLE"),
            Operation::GetVariable => write!(f, "GET_VARIABLE"),
            Operation::SetVariable => write!(f, "SET_VARIABLE"),
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
    use std::mem::size_of;

    use super::*;

    #[test]
    fn instruction_is_32_bits() {
        assert_eq!(size_of::<Instruction>(), 4);
    }
}
