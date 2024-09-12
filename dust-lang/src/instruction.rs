use std::fmt::{self, Display, Formatter};

use crate::{Chunk, Span};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Instruction {
    pub operation: Operation,
    pub to_register: u8,
    pub arguments: [u8; 2],
}

impl Instruction {
    pub fn decode(bits: u32) -> Instruction {
        let operation = Operation::from((bits >> 24) as u8);
        let to_register = ((bits >> 16) & 0xff) as u8;
        let arguments = [((bits >> 8) & 0xff) as u8, (bits & 0xff) as u8];

        Instruction {
            operation,
            to_register,
            arguments,
        }
    }

    pub fn encode(&self) -> u32 {
        let operation = u32::from(self.operation as u8);
        let to_register = u32::from(self.to_register);
        let arguments = u32::from(self.arguments[0]) << 8 | u32::from(self.arguments[1]);

        operation << 24 | to_register << 16 | arguments
    }

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
            Operation::Move => {
                format!(
                    "{:16} R({}) R({})",
                    self.operation.to_string(),
                    self.to_register,
                    self.arguments[0]
                )
            }
            Operation::Close => format!("{} R({})", self.operation, self.to_register),
            Operation::LoadConstant => {
                let constant_index = u16::from_le_bytes(self.arguments);
                let constant_display = match chunk.get_constant(constant_index, Span(0, 0)) {
                    Ok(value) => value.to_string(),
                    Err(error) => format!("{:?}", error),
                };

                format!(
                    "{:16} R({}) = C({}) {} ",
                    self.operation.to_string(),
                    self.to_register,
                    constant_index,
                    constant_display
                )
            }
            Operation::DeclareVariable => {
                let identifier_index = u16::from_le_bytes([self.arguments[0], self.arguments[1]]);

                format!(
                    "{:16} R[C({})] = R({})",
                    self.operation.to_string(),
                    identifier_index,
                    self.to_register
                )
            }
            Operation::GetVariable => {
                let identifier_index = u16::from_le_bytes([self.arguments[0], self.arguments[1]]);

                format!(
                    "{:16} R{} = R[I({})]",
                    self.operation.to_string(),
                    self.to_register,
                    identifier_index
                )
            }
            Operation::SetVariable => {
                let identifier_index = u16::from_le_bytes([self.arguments[0], self.arguments[1]]);

                format!(
                    "{:16}  R[C({})] = R({})",
                    self.operation.to_string(),
                    identifier_index,
                    self.to_register
                )
            }
            Operation::Add => {
                format!(
                    "{:16} R({}) = R({}) + R({})",
                    self.operation.to_string(),
                    self.to_register,
                    self.arguments[0],
                    self.arguments[1]
                )
            }
            Operation::Subtract => {
                format!(
                    "{:16} R({}) = R({}) - R({})",
                    self.operation.to_string(),
                    self.to_register,
                    self.arguments[0],
                    self.arguments[1]
                )
            }
            Operation::Multiply => {
                format!(
                    "{:16} R({}) = R({}) * R({})",
                    self.operation.to_string(),
                    self.to_register,
                    self.arguments[0],
                    self.arguments[1]
                )
            }
            Operation::Divide => {
                format!(
                    "{:16} R({}) = R({}) / R({})",
                    self.operation.to_string(),
                    self.to_register,
                    self.arguments[0],
                    self.arguments[1]
                )
            }
            Operation::Negate => {
                format!(
                    "{:16} R({}) = -R({})",
                    self.operation.to_string(),
                    self.to_register,
                    self.arguments[0]
                )
            }
            Operation::Return => {
                format!("{:16}", self.operation.to_string())
            }
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.operation {
            Operation::Move => {
                write!(
                    f,
                    "{:16} R({}) R({})",
                    self.operation.to_string(),
                    self.to_register,
                    self.arguments[0]
                )
            }
            Operation::Close => write!(f, "{} R({})", self.operation, self.to_register),
            Operation::LoadConstant => {
                let constant_index = u16::from_le_bytes(self.arguments);

                write!(
                    f,
                    "{:16} R({}) C({})",
                    self.operation.to_string(),
                    self.to_register,
                    constant_index
                )
            }
            Operation::DeclareVariable => {
                let identifier_index = u16::from_le_bytes([self.arguments[0], self.arguments[1]]);

                write!(
                    f,
                    "{:16} R[C({})] = R({})",
                    self.operation.to_string(),
                    identifier_index,
                    self.to_register
                )
            }
            Operation::GetVariable => {
                let identifier_index = u16::from_le_bytes([self.arguments[0], self.arguments[1]]);

                write!(
                    f,
                    "{:16} R{} = R[I({})]",
                    self.operation.to_string(),
                    self.to_register,
                    identifier_index
                )
            }
            Operation::SetVariable => {
                let identifier_index = u16::from_le_bytes([self.arguments[0], self.arguments[1]]);

                write!(
                    f,
                    "{:16}  R[C({})] = R({})",
                    self.operation.to_string(),
                    identifier_index,
                    self.to_register
                )
            }
            Operation::Add => {
                write!(
                    f,
                    "{:16} R({}) = R({}) + R({})",
                    self.operation.to_string(),
                    self.to_register,
                    self.arguments[0],
                    self.arguments[1]
                )
            }
            Operation::Subtract => {
                write!(
                    f,
                    "{:16} R({}) = R({}) - R({})",
                    self.operation.to_string(),
                    self.to_register,
                    self.arguments[0],
                    self.arguments[1]
                )
            }
            Operation::Multiply => {
                write!(
                    f,
                    "{:16} R({}) = R({}) * R({})",
                    self.operation.to_string(),
                    self.to_register,
                    self.arguments[0],
                    self.arguments[1]
                )
            }
            Operation::Divide => {
                write!(
                    f,
                    "{:16} R({}) = R({}) / R({})",
                    self.operation.to_string(),
                    self.to_register,
                    self.arguments[0],
                    self.arguments[1]
                )
            }
            Operation::Negate => {
                write!(
                    f,
                    "{:16} R({}) = -R({})",
                    self.operation.to_string(),
                    self.to_register,
                    self.arguments[0]
                )
            }
            Operation::Return => {
                write!(f, "{:16}", self.operation.to_string())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operation {
    // Stack manipulation
    Move = 0,
    Close = 1,

    // Constants
    LoadConstant = 2,

    // Variables
    DeclareVariable = 3,
    GetVariable = 4,
    SetVariable = 5,

    // Binary operations
    Add = 6,
    Subtract = 7,
    Multiply = 8,
    Divide = 9,

    // Unary operations
    Negate = 10,

    // Control flow
    Return = 11,
}

impl From<u8> for Operation {
    fn from(byte: u8) -> Self {
        match byte {
            0 => Operation::Move,
            1 => Operation::Close,
            2 => Operation::LoadConstant,
            3 => Operation::DeclareVariable,
            4 => Operation::GetVariable,
            5 => Operation::SetVariable,
            6 => Operation::Add,
            7 => Operation::Subtract,
            8 => Operation::Multiply,
            9 => Operation::Divide,
            10 => Operation::Negate,
            _ => Operation::Return,
        }
    }
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
