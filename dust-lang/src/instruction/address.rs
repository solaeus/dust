use std::fmt::{self, Display, Formatter};

use super::TypeCode;

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Address {
    pub index: u16,
    pub kind: AddressKind,
}

impl Address {
    pub fn new(index: u16, kind: AddressKind) -> Self {
        Address { index, kind }
    }

    pub fn as_type_code(&self) -> TypeCode {
        match self.kind {
            AddressKind::BOOLEAN_MEMORY | AddressKind::BOOLEAN_REGISTER => TypeCode::BOOLEAN,
            AddressKind::BYTE_MEMORY | AddressKind::BYTE_REGISTER => TypeCode::BYTE,
            AddressKind::CHARACTER_CONSTANT
            | AddressKind::CHARACTER_MEMORY
            | AddressKind::CHARACTER_REGISTER => TypeCode::CHARACTER,
            AddressKind::FLOAT_CONSTANT
            | AddressKind::FLOAT_MEMORY
            | AddressKind::FLOAT_REGISTER => TypeCode::FLOAT,
            AddressKind::INTEGER_CONSTANT
            | AddressKind::INTEGER_MEMORY
            | AddressKind::INTEGER_REGISTER => TypeCode::INTEGER,
            AddressKind::STRING_CONSTANT
            | AddressKind::STRING_MEMORY
            | AddressKind::STRING_REGISTER => TypeCode::STRING,
            AddressKind::LIST_MEMORY | AddressKind::LIST_REGISTER => TypeCode::LIST,
            AddressKind::FUNCTION_SELF
            | AddressKind::FUNCTION_MEMORY
            | AddressKind::FUNCTION_REGISTER => TypeCode::FUNCTION,
            unknown => unreachable!("Invalid OperandKind: {}", unknown.0),
        }
    }

    pub fn is_constant(&self) -> bool {
        matches!(
            self.kind,
            AddressKind::CHARACTER_CONSTANT
                | AddressKind::FLOAT_CONSTANT
                | AddressKind::INTEGER_CONSTANT
                | AddressKind::STRING_CONSTANT
        )
    }

    pub fn is_register(&self) -> bool {
        matches!(
            self.kind,
            AddressKind::BOOLEAN_REGISTER
                | AddressKind::BYTE_REGISTER
                | AddressKind::CHARACTER_REGISTER
                | AddressKind::FLOAT_REGISTER
                | AddressKind::INTEGER_REGISTER
                | AddressKind::STRING_REGISTER
                | AddressKind::LIST_REGISTER
                | AddressKind::FUNCTION_REGISTER
        )
    }
}

impl Default for Address {
    fn default() -> Self {
        Address {
            index: 0,
            kind: AddressKind(0),
        }
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let index = self.index;

        match self.kind {
            AddressKind::BOOLEAN_MEMORY => write!(f, "M_BOOL_{index}"),
            AddressKind::BOOLEAN_REGISTER => write!(f, "R_BOOL_{index}"),
            AddressKind::BYTE_MEMORY => write!(f, "M_BYTE_{index}"),
            AddressKind::BYTE_REGISTER => write!(f, "R_BYTE_{index}"),
            AddressKind::CHARACTER_CONSTANT => write!(f, "C_CHAR_{index}"),
            AddressKind::CHARACTER_MEMORY => write!(f, "M_CHAR_{index}"),
            AddressKind::CHARACTER_REGISTER => write!(f, "R_CHAR_{index}"),
            AddressKind::FLOAT_CONSTANT => write!(f, "C_FLOAT_{index}"),
            AddressKind::FLOAT_MEMORY => write!(f, "M_FLOAT_{index}"),
            AddressKind::FLOAT_REGISTER => write!(f, "R_FLOAT_{index}"),
            AddressKind::INTEGER_CONSTANT => write!(f, "C_INT_{index}"),
            AddressKind::INTEGER_MEMORY => write!(f, "M_INT_{index}"),
            AddressKind::INTEGER_REGISTER => write!(f, "R_INT_{index}"),
            AddressKind::STRING_CONSTANT => write!(f, "C_STR_{index}"),
            AddressKind::STRING_MEMORY => write!(f, "M_STR_{index}"),
            AddressKind::STRING_REGISTER => write!(f, "R_STR_{index}"),
            AddressKind::LIST_MEMORY => write!(f, "M_LIST_{index}"),
            AddressKind::LIST_REGISTER => write!(f, "R_LIST_{index}"),
            AddressKind::FUNCTION_MEMORY => write!(f, "M_FN_{index}"),
            AddressKind::FUNCTION_REGISTER => write!(f, "R_FN_{index}"),
            AddressKind::FUNCTION_SELF => write!(f, "SELF"),
            _ => unreachable!("Invalid OperandKind"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AddressKind(pub u8);

impl AddressKind {
    pub const BOOLEAN_MEMORY: AddressKind = AddressKind(0);
    pub const BOOLEAN_REGISTER: AddressKind = AddressKind(1);

    pub const BYTE_MEMORY: AddressKind = AddressKind(2);
    pub const BYTE_REGISTER: AddressKind = AddressKind(3);

    pub const CHARACTER_CONSTANT: AddressKind = AddressKind(4);
    pub const CHARACTER_MEMORY: AddressKind = AddressKind(5);
    pub const CHARACTER_REGISTER: AddressKind = AddressKind(6);

    pub const FLOAT_CONSTANT: AddressKind = AddressKind(7);
    pub const FLOAT_MEMORY: AddressKind = AddressKind(8);
    pub const FLOAT_REGISTER: AddressKind = AddressKind(9);

    pub const INTEGER_CONSTANT: AddressKind = AddressKind(10);
    pub const INTEGER_MEMORY: AddressKind = AddressKind(11);
    pub const INTEGER_REGISTER: AddressKind = AddressKind(12);

    pub const STRING_CONSTANT: AddressKind = AddressKind(13);
    pub const STRING_MEMORY: AddressKind = AddressKind(14);
    pub const STRING_REGISTER: AddressKind = AddressKind(15);

    pub const LIST_MEMORY: AddressKind = AddressKind(16);
    pub const LIST_REGISTER: AddressKind = AddressKind(17);

    pub const FUNCTION_MEMORY: AddressKind = AddressKind(18);
    pub const FUNCTION_PROTOTYPE: AddressKind = AddressKind(19);
    pub const FUNCTION_REGISTER: AddressKind = AddressKind(20);
    pub const FUNCTION_SELF: AddressKind = AddressKind(21);
}
