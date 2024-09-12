use crate::{Chunk, Span};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Instruction {
    opcode: OpCode,
    to_register: u8,
    arguments: [u8; 2],
}

impl Instruction {
    pub fn r#move(to_register: u8, from_register: u8) -> Instruction {
        Instruction {
            opcode: OpCode::Move,
            to_register,
            arguments: [from_register, 0],
        }
    }

    pub fn close(to_register: u8) -> Instruction {
        Instruction {
            opcode: OpCode::Close,
            to_register,
            arguments: [0, 0],
        }
    }

    pub fn load_constant(to_register: u8, constant_index: u16) -> Instruction {
        Instruction {
            opcode: OpCode::LoadConstant,
            to_register,
            arguments: constant_index.to_le_bytes(),
        }
    }

    pub fn declare_variable(to_register: u8, variable_index: u16) -> Instruction {
        Instruction {
            opcode: OpCode::DeclareVariable,
            to_register,
            arguments: variable_index.to_le_bytes(),
        }
    }

    pub fn get_variable(to_register: u8, variable_index: u16) -> Instruction {
        Instruction {
            opcode: OpCode::GetVariable,
            to_register,
            arguments: variable_index.to_le_bytes(),
        }
    }

    pub fn set_variable(from_register: u8, variable_index: u16) -> Instruction {
        Instruction {
            opcode: OpCode::SetVariable,
            to_register: from_register,
            arguments: variable_index.to_le_bytes(),
        }
    }

    pub fn add(to_register: u8, left_register: u8, right_register: u8) -> Instruction {
        Instruction {
            opcode: OpCode::Add,
            to_register,
            arguments: [left_register, right_register],
        }
    }

    pub fn subtract(to_register: u8, left_register: u8, right_register: u8) -> Instruction {
        Instruction {
            opcode: OpCode::Subtract,
            to_register,
            arguments: [left_register, right_register],
        }
    }

    pub fn multiply(to_register: u8, left_register: u8, right_register: u8) -> Instruction {
        Instruction {
            opcode: OpCode::Multiply,
            to_register,
            arguments: [left_register, right_register],
        }
    }

    pub fn divide(to_register: u8, left_register: u8, right_register: u8) -> Instruction {
        Instruction {
            opcode: OpCode::Divide,
            to_register,
            arguments: [left_register, right_register],
        }
    }

    pub fn negate(to_register: u8, from_register: u8) -> Instruction {
        Instruction {
            opcode: OpCode::Negate,
            to_register,
            arguments: [from_register, 0],
        }
    }

    pub fn r#return() -> Instruction {
        Instruction {
            opcode: OpCode::Return,
            to_register: 0,
            arguments: [0, 0],
        }
    }

    pub fn disassemble(&self, chunk: &Chunk, offset: usize) -> String {
        match self.opcode {
            OpCode::Move => format!(
                "{:04} MOVE R{} R{}",
                offset, self.to_register, self.arguments[0]
            ),
            OpCode::Close => {
                format!("{:04} CLOSE R{}", offset, self.to_register)
            }
            OpCode::LoadConstant => {
                let constant_index = u16::from_le_bytes(self.arguments);
                let constant_display = match chunk.get_constant(constant_index, Span(0, 0)) {
                    Ok(value) => value.to_string(),
                    Err(error) => format!("{:?}", error),
                };

                format!(
                    "{:04} LOAD_CONSTANT R{} C{} {}",
                    offset, self.to_register, constant_index, constant_display
                )
            }
            OpCode::DeclareVariable => {
                let variable_index = u16::from_le_bytes([self.arguments[0], self.arguments[1]]);

                format!(
                    "{:04} DECLARE_VARIABLE V{} R{}",
                    offset, variable_index, self.to_register
                )
            }
            OpCode::GetVariable => {
                let variable_index = u16::from_le_bytes([self.arguments[0], self.arguments[1]]);

                format!(
                    "{:04} GET_VARIABLE R{} V{}",
                    offset, self.to_register, variable_index
                )
            }
            OpCode::SetVariable => {
                let variable_index = u16::from_le_bytes([self.arguments[0], self.arguments[1]]);

                format!(
                    "{:04} SET_VARIABLE V{} R{}",
                    offset, variable_index, self.to_register
                )
            }
            OpCode::Add => format!(
                "{:04} ADD R{} = R{} + R{}",
                offset, self.to_register, self.arguments[0], self.arguments[1]
            ),
            OpCode::Subtract => format!(
                "{:04} SUBTRACT R{} = R{} - R{}",
                offset, self.to_register, self.arguments[0], self.arguments[1]
            ),
            OpCode::Multiply => format!(
                "{:04} MULTIPLY R{} = R{} * R{}",
                offset, self.to_register, self.arguments[0], self.arguments[1]
            ),
            OpCode::Divide => format!(
                "{:04} DIVIDE R{} = R{} / R{}",
                offset, self.to_register, self.arguments[0], self.arguments[1]
            ),
            OpCode::Negate => format!(
                "{:04} NEGATE R{} = !R{}",
                offset, self.to_register, self.arguments[0]
            ),
            OpCode::Return => format!("{:04} RETURN", offset),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum OpCode {
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

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    #[test]
    fn instruction_is_32_bits() {
        assert_eq!(size_of::<Instruction>(), 4);
    }
}
