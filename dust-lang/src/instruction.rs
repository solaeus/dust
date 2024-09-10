use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::Chunk;

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
