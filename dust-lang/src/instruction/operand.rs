use std::fmt::{self, Display, Formatter};

use super::TypeCode;

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Operand {
    pub index: u16,
    pub kind: OperandKind,
}

impl Operand {
    pub fn as_type_code(&self) -> TypeCode {
        match self.kind {
            OperandKind::BOOLEAN_MEMORY | OperandKind::BOOLEAN_REGISTER => TypeCode::BOOLEAN,
            OperandKind::BYTE_MEMORY | OperandKind::BYTE_REGISTER => TypeCode::BYTE,
            OperandKind::CHARACTER_CONSTANT
            | OperandKind::CHARACTER_MEMORY
            | OperandKind::CHARACTER_REGISTER => TypeCode::CHARACTER,
            OperandKind::FLOAT_CONSTANT
            | OperandKind::FLOAT_MEMORY
            | OperandKind::FLOAT_REGISTER => TypeCode::FLOAT,
            OperandKind::INTEGER_CONSTANT
            | OperandKind::INTEGER_MEMORY
            | OperandKind::INTEGER_REGISTER => TypeCode::INTEGER,
            OperandKind::STRING_CONSTANT
            | OperandKind::STRING_MEMORY
            | OperandKind::STRING_REGISTER => TypeCode::STRING,
            OperandKind::LIST_MEMORY | OperandKind::LIST_REGISTER => TypeCode::LIST,
            OperandKind::FUNCTION_SELF
            | OperandKind::FUNCTION_MEMORY
            | OperandKind::FUNCTION_REGISTER => TypeCode::FUNCTION,
            unknown => unreachable!("Invalid OperandKind: {}", unknown.0),
        }
    }

    pub fn is_constant(&self) -> bool {
        matches!(
            self.kind,
            OperandKind::CHARACTER_CONSTANT
                | OperandKind::FLOAT_CONSTANT
                | OperandKind::INTEGER_CONSTANT
                | OperandKind::STRING_CONSTANT
        )
    }

    pub fn is_register(&self) -> bool {
        matches!(
            self.kind,
            OperandKind::BOOLEAN_REGISTER
                | OperandKind::BYTE_REGISTER
                | OperandKind::CHARACTER_REGISTER
                | OperandKind::FLOAT_REGISTER
                | OperandKind::INTEGER_REGISTER
                | OperandKind::STRING_REGISTER
                | OperandKind::LIST_REGISTER
                | OperandKind::FUNCTION_REGISTER
        )
    }

    pub fn as_index_and_constant_flag(&self) -> (u16, bool) {
        (self.index, self.is_constant())
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let index = self.index;

        match self.kind {
            OperandKind::BOOLEAN_MEMORY => write!(f, "M_BOOL_{index}"),
            OperandKind::BOOLEAN_REGISTER => write!(f, "R_BOOL_{index}"),
            OperandKind::BYTE_MEMORY => write!(f, "M_BYTE_{index}"),
            OperandKind::BYTE_REGISTER => write!(f, "R_BYTE_{index}"),
            OperandKind::CHARACTER_CONSTANT => write!(f, "C_CHAR_{index}"),
            OperandKind::CHARACTER_MEMORY => write!(f, "M_CHAR_{index}"),
            OperandKind::CHARACTER_REGISTER => write!(f, "R_CHAR_{index}"),
            OperandKind::FLOAT_CONSTANT => write!(f, "C_FLOAT_{index}"),
            OperandKind::FLOAT_MEMORY => write!(f, "M_FLOAT_{index}"),
            OperandKind::FLOAT_REGISTER => write!(f, "R_FLOAT_{index}"),
            OperandKind::INTEGER_CONSTANT => write!(f, "C_INT_{index}"),
            OperandKind::INTEGER_MEMORY => write!(f, "M_INT_{index}"),
            OperandKind::INTEGER_REGISTER => write!(f, "R_INT_{index}"),
            OperandKind::STRING_CONSTANT => write!(f, "C_STR_{index}"),
            OperandKind::STRING_MEMORY => write!(f, "M_STR_{index}"),
            OperandKind::STRING_REGISTER => write!(f, "R_STR_{index}"),
            OperandKind::LIST_MEMORY => write!(f, "M_LIST_{index}"),
            OperandKind::LIST_REGISTER => write!(f, "R_LIST_{index}"),
            OperandKind::FUNCTION_MEMORY => write!(f, "M_FN_{index}"),
            OperandKind::FUNCTION_REGISTER => write!(f, "R_FN_{index}"),
            OperandKind::FUNCTION_SELF => write!(f, "SELF"),
            _ => unreachable!("Invalid OperandKind"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OperandKind(pub u8);

impl OperandKind {
    pub const BOOLEAN_MEMORY: OperandKind = OperandKind(0);
    pub const BOOLEAN_REGISTER: OperandKind = OperandKind(1);

    pub const BYTE_MEMORY: OperandKind = OperandKind(2);
    pub const BYTE_REGISTER: OperandKind = OperandKind(3);

    pub const CHARACTER_CONSTANT: OperandKind = OperandKind(4);
    pub const CHARACTER_MEMORY: OperandKind = OperandKind(5);
    pub const CHARACTER_REGISTER: OperandKind = OperandKind(6);

    pub const FLOAT_CONSTANT: OperandKind = OperandKind(7);
    pub const FLOAT_MEMORY: OperandKind = OperandKind(8);
    pub const FLOAT_REGISTER: OperandKind = OperandKind(9);

    pub const INTEGER_CONSTANT: OperandKind = OperandKind(10);
    pub const INTEGER_MEMORY: OperandKind = OperandKind(11);
    pub const INTEGER_REGISTER: OperandKind = OperandKind(12);

    pub const STRING_CONSTANT: OperandKind = OperandKind(13);
    pub const STRING_MEMORY: OperandKind = OperandKind(14);
    pub const STRING_REGISTER: OperandKind = OperandKind(15);

    pub const LIST_MEMORY: OperandKind = OperandKind(16);
    pub const LIST_REGISTER: OperandKind = OperandKind(17);

    pub const FUNCTION_MEMORY: OperandKind = OperandKind(18);
    pub const FUNCTION_REGISTER: OperandKind = OperandKind(19);
    pub const FUNCTION_SELF: OperandKind = OperandKind(20);
}
