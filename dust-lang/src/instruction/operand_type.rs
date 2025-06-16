use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

use super::Operation;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OperandType(pub u8);

impl OperandType {
    // Operand fields are meaningless
    pub const NONE: OperandType = OperandType(0);

    // One or two operands of the same type
    pub const BOOLEAN: OperandType = OperandType(1);
    pub const BYTE: OperandType = OperandType(2);
    pub const CHARACTER: OperandType = OperandType(3);
    pub const FLOAT: OperandType = OperandType(4);
    pub const INTEGER: OperandType = OperandType(5);
    pub const STRING: OperandType = OperandType(6);
    pub const LIST: OperandType = OperandType(7);
    pub const FUNCTION: OperandType = OperandType(8);

    // Two operands of different types
    pub const CHARACTER_STRING: OperandType = OperandType(9);
    pub const STRING_CHARACTER: OperandType = OperandType(10);
}

impl Debug for OperandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for OperandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::NONE => write!(f, "NONE"),
            Self::BOOLEAN => write!(f, "BOOLEAN"),
            Self::BYTE => write!(f, "BYTE"),
            Self::CHARACTER => write!(f, "CHARACTER"),
            Self::FLOAT => write!(f, "FLOAT"),
            Self::INTEGER => write!(f, "INTEGER"),
            Self::STRING => write!(f, "STRING"),
            Self::LIST => write!(f, "LIST"),
            Self::FUNCTION => write!(f, "FUNCTION"),
            Self::CHARACTER_STRING => write!(f, "CHARACTER_STRING"),
            Self::STRING_CHARACTER => write!(f, "STRING_CHARACTER"),
            _ => unreachable!(),
        }
    }
}

impl OperandType {
    pub fn invalid_panic(&self, operation: Operation) -> ! {
        panic!("Operand type {self} for {operation} instruction is invalid");
    }
}
