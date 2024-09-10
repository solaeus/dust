use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::Chunk;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Instruction {
    Constant = 0,
    Return = 1,
    Pop = 2,

    // Variables
    DefineVariableRuntime = 3,
    DefineVariableConstant = 4,
    GetVariable = 5,
    SetVariable = 6,

    // Unary
    Negate = 7,
    Not = 8,

    // Binary
    Add = 9,
    Subtract = 10,
    Multiply = 11,
    Divide = 12,
    Greater = 13,
    Less = 14,
    GreaterEqual = 15,
    LessEqual = 16,
    Equal = 17,
    NotEqual = 18,
    And = 19,
    Or = 20,
}

impl Instruction {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Instruction::Constant),
            1 => Some(Instruction::Return),
            2 => Some(Instruction::Pop),
            3 => Some(Instruction::DefineVariableRuntime),
            4 => Some(Instruction::DefineVariableConstant),
            5 => Some(Instruction::GetVariable),
            6 => Some(Instruction::SetVariable),
            7 => Some(Instruction::Negate),
            8 => Some(Instruction::Not),
            9 => Some(Instruction::Add),
            10 => Some(Instruction::Subtract),
            11 => Some(Instruction::Multiply),
            12 => Some(Instruction::Divide),
            13 => Some(Instruction::Greater),
            14 => Some(Instruction::Less),
            15 => Some(Instruction::GreaterEqual),
            16 => Some(Instruction::LessEqual),
            17 => Some(Instruction::Equal),
            18 => Some(Instruction::NotEqual),
            19 => Some(Instruction::And),
            20 => Some(Instruction::Or),
            _ => None,
        }
    }

    pub fn disassemble(&self, chunk: &Chunk, offset: usize) -> String {
        match self {
            Instruction::Constant => {
                let (argument, _) = chunk.get_code(offset + 1).unwrap();
                let value_display = chunk
                    .get_constant(*argument)
                    .map(|value| value.to_string())
                    .unwrap_or_else(|error| error.to_string());

                format!("CONSTANT {argument} {value_display}")
            }
            Instruction::Return => "RETURN".to_string(),
            Instruction::Pop => "POP".to_string(),

            // Variables
            Instruction::DefineVariableRuntime => "DEFINE_VARIABLE_RUNTIME".to_string(),
            Instruction::DefineVariableConstant => {
                let (argument, _) = chunk.get_code(offset + 1).unwrap();
                let identifier_display = match chunk.get_identifier(*argument) {
                    Ok(identifier) => identifier.to_string(),
                    Err(error) => error.to_string(),
                };

                format!("DEFINE_VARIABLE_CONSTANT {argument} {identifier_display}")
            }
            Instruction::GetVariable => {
                let (argument, _) = chunk.get_code(offset + 1).unwrap();
                let identifier_display = match chunk.get_identifier(*argument) {
                    Ok(identifier) => identifier.to_string(),
                    Err(error) => error.to_string(),
                };

                format!("GET_VARIABLE {argument} {identifier_display}")
            }

            Instruction::SetVariable => {
                let (argument, _) = chunk.get_code(offset + 1).unwrap();
                let identifier_display = match chunk.get_identifier(*argument) {
                    Ok(identifier) => identifier.to_string(),
                    Err(error) => error.to_string(),
                };

                format!("SET_VARIABLE {identifier_display}")
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

impl From<Instruction> for u8 {
    fn from(instruction: Instruction) -> Self {
        instruction as u8
    }
}
