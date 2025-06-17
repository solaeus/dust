use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

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
    pub const MAP: OperandType = OperandType(8);
    pub const FUNCTION: OperandType = OperandType(9);

    // Two operands of different types
    pub const CHARACTER_STRING: OperandType = OperandType(10);
    pub const STRING_CHARACTER: OperandType = OperandType(11);

    // List operands
    pub const LIST_BOOLEAN: OperandType = OperandType(12);
    pub const LIST_BYTE: OperandType = OperandType(13);
    pub const LIST_CHARACTER: OperandType = OperandType(14);
    pub const LIST_FLOAT: OperandType = OperandType(15);
    pub const LIST_INTEGER: OperandType = OperandType(16);
    pub const LIST_STRING: OperandType = OperandType(17);
    pub const LIST_LIST: OperandType = OperandType(18);
    pub const LIST_MAP: OperandType = OperandType(19);
    pub const LIST_FUNCTION: OperandType = OperandType(20);
}

impl OperandType {
    pub fn is_list(&self) -> bool {
        matches!(
            *self,
            Self::LIST_BOOLEAN
                | Self::LIST_BYTE
                | Self::LIST_CHARACTER
                | Self::LIST_FLOAT
                | Self::LIST_INTEGER
                | Self::LIST_STRING
                | Self::LIST_LIST
                | Self::LIST_MAP
                | Self::LIST_FUNCTION
        )
    }

    pub fn list_item_type(&self) -> Option<Self> {
        match *self {
            Self::LIST_BOOLEAN => Some(Self::BOOLEAN),
            Self::LIST_BYTE => Some(Self::BYTE),
            Self::LIST_CHARACTER => Some(Self::CHARACTER),
            Self::LIST_FLOAT => Some(Self::FLOAT),
            Self::LIST_INTEGER => Some(Self::INTEGER),
            Self::LIST_STRING => Some(Self::STRING),
            Self::LIST_LIST => Some(Self::LIST),
            Self::LIST_MAP => Some(Self::MAP),
            Self::LIST_FUNCTION => Some(Self::FUNCTION),
            _ => None,
        }
    }

    pub fn destination_type(&self) -> Self {
        match *self {
            Self::CHARACTER_STRING | Self::STRING_CHARACTER => OperandType::STRING,
            Self::LIST_BOOLEAN
            | Self::LIST_BYTE
            | Self::LIST_CHARACTER
            | Self::LIST_FLOAT
            | Self::LIST_INTEGER
            | Self::LIST_STRING
            | Self::LIST_LIST
            | Self::LIST_MAP
            | Self::LIST_FUNCTION => OperandType::LIST,
            _ => *self,
        }
    }

    pub fn b_type(&self) -> Self {
        match *self {
            Self::CHARACTER_STRING => OperandType::CHARACTER,
            Self::STRING_CHARACTER => OperandType::STRING,
            Self::LIST_BOOLEAN => OperandType::BOOLEAN,
            Self::LIST_BYTE => OperandType::BYTE,
            Self::LIST_CHARACTER => OperandType::CHARACTER,
            Self::LIST_FLOAT => OperandType::FLOAT,
            Self::LIST_INTEGER => OperandType::INTEGER,
            Self::LIST_STRING => OperandType::STRING,
            Self::LIST_LIST => OperandType::LIST,
            Self::LIST_MAP => OperandType::MAP,
            Self::LIST_FUNCTION => OperandType::FUNCTION,
            _ => *self,
        }
    }

    pub fn c_type(&self) -> Self {
        match *self {
            Self::CHARACTER_STRING => OperandType::STRING,
            Self::STRING_CHARACTER => OperandType::CHARACTER,
            Self::LIST_BOOLEAN => OperandType::BOOLEAN,
            Self::LIST_BYTE => OperandType::BYTE,
            Self::LIST_CHARACTER => OperandType::CHARACTER,
            Self::LIST_FLOAT => OperandType::FLOAT,
            Self::LIST_INTEGER => OperandType::INTEGER,
            Self::LIST_STRING => OperandType::STRING,
            Self::LIST_LIST => OperandType::LIST,
            Self::LIST_MAP => OperandType::MAP,
            Self::LIST_FUNCTION => OperandType::FUNCTION,
            _ => *self,
        }
    }
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
            Self::MAP => write!(f, "MAP"),
            Self::FUNCTION => write!(f, "FUNCTION"),
            Self::CHARACTER_STRING => write!(f, "CHARACTER_STRING"),
            Self::STRING_CHARACTER => write!(f, "STRING_CHARACTER"),
            Self::LIST_BOOLEAN => write!(f, "LIST_BOOLEAN"),
            Self::LIST_BYTE => write!(f, "LIST_BYTE"),
            Self::LIST_CHARACTER => write!(f, "LIST_CHARACTER"),
            Self::LIST_FLOAT => write!(f, "LIST_FLOAT"),
            Self::LIST_INTEGER => write!(f, "LIST_INTEGER"),
            Self::LIST_STRING => write!(f, "LIST_STRING"),
            Self::LIST_LIST => write!(f, "LIST_LIST"),
            Self::LIST_FUNCTION => write!(f, "LIST_FUNCTION"),
            invalid => write!(f, "INVALID_OPERAND_TYPE({})", invalid.0),
        }
    }
}

#[macro_export]
macro_rules! invalid_operand_type_panic {
    ($invalid: expr, $operation: expr) => {
        panic!(
            "Operand type {} is invalid for {} instruction. This is a bug in the compiler or VM.",
            $invalid, $operation,
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_operation_types_are_safe() {
        for i in 0..=17 {
            let operand_type = OperandType(i);

            let _ = operand_type.destination_type();
            let _ = operand_type.b_type();
            let _ = operand_type.c_type();
        }
    }
}
