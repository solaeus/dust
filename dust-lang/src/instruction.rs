use std::fmt::{self, Display, Formatter};

use crate::{Chunk, Span};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Instruction {
    pub operation: Operation,
    pub destination: u8,
    pub arguments: [u8; 2],
}

impl Instruction {
    pub fn decode(bits: u32) -> Instruction {
        let operation = Operation::from((bits >> 24) as u8);
        let to_register = ((bits >> 16) & 0xff) as u8;
        let arguments = [((bits >> 8) & 0xff) as u8, (bits & 0xff) as u8];

        Instruction {
            operation,
            destination: to_register,
            arguments,
        }
    }

    pub fn encode(&self) -> u32 {
        let operation = self.operation as u8 as u32;
        let to_register = self.destination as u32;
        let arguments = (self.arguments[0] as u32) << 8 | (self.arguments[1] as u32);

        operation << 24 | to_register << 16 | arguments
    }

    pub fn r#move(to_register: u8, from_register: u8) -> Instruction {
        Instruction {
            operation: Operation::Move,
            destination: to_register,
            arguments: [from_register, 0],
        }
    }

    pub fn close(to_register: u8) -> Instruction {
        Instruction {
            operation: Operation::Close,
            destination: to_register,
            arguments: [0, 0],
        }
    }

    pub fn load_constant(to_register: u8, constant_index: u16) -> Instruction {
        Instruction {
            operation: Operation::LoadConstant,
            destination: to_register,
            arguments: constant_index.to_le_bytes(),
        }
    }

    pub fn declare_local(to_register: u8, variable_index: u16) -> Instruction {
        Instruction {
            operation: Operation::DeclareLocal,
            destination: to_register,
            arguments: variable_index.to_le_bytes(),
        }
    }

    pub fn get_local(to_register: u8, variable_index: u16) -> Instruction {
        Instruction {
            operation: Operation::GetLocal,
            destination: to_register,
            arguments: variable_index.to_le_bytes(),
        }
    }

    pub fn set_local(from_register: u8, variable_index: u16) -> Instruction {
        Instruction {
            operation: Operation::SetLocal,
            destination: from_register,
            arguments: variable_index.to_le_bytes(),
        }
    }

    pub fn add(to_register: u8, left_register: u8, right_register: u8) -> Instruction {
        Instruction {
            operation: Operation::Add,
            destination: to_register,
            arguments: [left_register, right_register],
        }
    }

    pub fn subtract(to_register: u8, left_register: u8, right_register: u8) -> Instruction {
        Instruction {
            operation: Operation::Subtract,
            destination: to_register,
            arguments: [left_register, right_register],
        }
    }

    pub fn multiply(to_register: u8, left_register: u8, right_register: u8) -> Instruction {
        Instruction {
            operation: Operation::Multiply,
            destination: to_register,
            arguments: [left_register, right_register],
        }
    }

    pub fn divide(to_register: u8, left_register: u8, right_register: u8) -> Instruction {
        Instruction {
            operation: Operation::Divide,
            destination: to_register,
            arguments: [left_register, right_register],
        }
    }

    pub fn negate(to_register: u8, from_register: u8) -> Instruction {
        Instruction {
            operation: Operation::Negate,
            destination: to_register,
            arguments: [from_register, 0],
        }
    }

    pub fn r#return() -> Instruction {
        Instruction {
            operation: Operation::Return,
            destination: 0,
            arguments: [0, 0],
        }
    }

    pub fn disassemble(&self, chunk: &Chunk) -> String {
        let mut disassembled = format!("{:16} ", self.operation.to_string());

        if let Some(info) = self.disassembly_info(Some(chunk)) {
            disassembled.push_str(&info);
        }

        disassembled
    }

    pub fn disassembly_info(&self, chunk: Option<&Chunk>) -> Option<String> {
        let info = match self.operation {
            Operation::Move => {
                format!("R({}) = R({})", self.destination, self.arguments[0])
            }
            Operation::Close => format!("R({})", self.destination),
            Operation::LoadConstant => {
                let constant_index = u16::from_le_bytes(self.arguments) as usize;

                if let Some(chunk) = chunk {
                    match chunk.get_constant(constant_index, Span(0, 0)) {
                        Ok(value) => {
                            format!("R({}) = C({}) {}", self.destination, constant_index, value)
                        }
                        Err(error) => format!(
                            "R({}) = C({}) {:?}",
                            self.destination, constant_index, error
                        ),
                    }
                } else {
                    format!("R({}) = C({})", self.destination, constant_index)
                }
            }
            Operation::DeclareLocal => {
                let local_index = u16::from_le_bytes([self.arguments[0], self.arguments[1]]);
                let identifier_display = if let Some(chunk) = chunk {
                    match chunk.get_identifier(local_index as usize) {
                        Some(identifier) => identifier.to_string(),
                        None => "???".to_string(),
                    }
                } else {
                    "???".to_string()
                };

                format!(
                    "L({}) = R({}) {}",
                    local_index, self.destination, identifier_display
                )
            }
            Operation::GetLocal => {
                let local_index = u16::from_le_bytes([self.arguments[0], self.arguments[1]]);

                format!("R({}) = L({})", self.destination, local_index)
            }
            Operation::SetLocal => {
                let local_index = u16::from_le_bytes([self.arguments[0], self.arguments[1]]);
                let identifier_display = if let Some(chunk) = chunk {
                    match chunk.get_identifier(local_index as usize) {
                        Some(identifier) => identifier.to_string(),
                        None => "???".to_string(),
                    }
                } else {
                    "???".to_string()
                };

                format!(
                    "L({}) = R({}) {}",
                    local_index, self.destination, identifier_display
                )
            }
            Operation::Add => {
                format!(
                    "R({}) = RC({}) + RC({})",
                    self.destination, self.arguments[0], self.arguments[1]
                )
            }
            Operation::Subtract => {
                format!(
                    "R({}) = RC({}) - RC({})",
                    self.destination, self.arguments[0], self.arguments[1]
                )
            }
            Operation::Multiply => {
                format!(
                    "R({}) = RC({}) * RC({})",
                    self.destination, self.arguments[0], self.arguments[1]
                )
            }
            Operation::Divide => {
                format!(
                    "R({}) = RC({}) / RC({})",
                    self.destination, self.arguments[0], self.arguments[1]
                )
            }
            Operation::Negate => {
                format!("R({}) = -RC({})", self.destination, self.arguments[0])
            }
            Operation::Return => return None,
        };

        Some(info)
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(info) = self.disassembly_info(None) {
            write!(f, "{} {}", self.operation, info)
        } else {
            write!(f, "{}", self.operation)
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
    DeclareLocal = 3,
    GetLocal = 4,
    SetLocal = 5,

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
            3 => Operation::DeclareLocal,
            4 => Operation::GetLocal,
            5 => Operation::SetLocal,
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
    use std::mem::size_of;

    use super::*;

    #[test]
    fn instruction_is_32_bits() {
        assert_eq!(size_of::<Instruction>(), 4);
    }
}
