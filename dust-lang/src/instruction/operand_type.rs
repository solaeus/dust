/// One-byte representation of a value type.
use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

enum Yo {
    X,
    Y,
    Z,
}

const X: u8 = Yo::X as u8;

/// One-byte representation of a value type.
///
/// This type is primarily used for encoding the types of operands in instructions, but it is also
/// useful whenever a compact representation of a type is needed. However, it can only represent
/// None, scalar types and shallow composite types. Instead, the user-facing API uses [Type][] and
/// the compiler uses [TypeResolver][].
///
/// [Type]: crate::r#type::Type
/// [TypeResolver]: crate::compiler::type_resolver::TypeResolver
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
    pub const MAP: OperandType = OperandType(8);
    pub const FUNCTION: OperandType = OperandType(9);

    // Two operands of different types
    pub const CHARACTER_STRING: OperandType = OperandType(10);
    pub const STRING_CHARACTER: OperandType = OperandType(11);

    // Array operands
    pub const ARRAY_BOOLEAN: OperandType = OperandType(12);
    pub const ARRAY_BYTE: OperandType = OperandType(13);
    pub const ARRAY_CHARACTER: OperandType = OperandType(14);
    pub const ARRAY_FLOAT: OperandType = OperandType(15);
    pub const ARRAY_INTEGER: OperandType = OperandType(16);
    pub const ARRAY_STRING: OperandType = OperandType(17);
    pub const ARRAY_FUNCTION: OperandType = OperandType(20);
    pub const ARRAY_ARRAY: OperandType = OperandType(31);
    pub const ARRAY_LIST: OperandType = OperandType(18);
    pub const ARRAY_MAP: OperandType = OperandType(19);

    // List operands
    pub const LIST_BOOLEAN: OperandType = OperandType(21);
    pub const LIST_BYTE: OperandType = OperandType(22);
    pub const LIST_CHARACTER: OperandType = OperandType(23);
    pub const LIST_FLOAT: OperandType = OperandType(24);
    pub const LIST_INTEGER: OperandType = OperandType(25);
    pub const LIST_STRING: OperandType = OperandType(26);
    pub const LIST_FUNCTION: OperandType = OperandType(29);
    pub const LIST_ARRAY: OperandType = OperandType(30);
    pub const LIST_MAP: OperandType = OperandType(28);
    pub const LIST_LIST: OperandType = OperandType(27);
}

impl OperandType {
    pub fn is_scalar(&self) -> bool {
        matches!(
            *self,
            Self::BOOLEAN
                | Self::BYTE
                | Self::CHARACTER
                | Self::FLOAT
                | Self::INTEGER
                | Self::FUNCTION
        )
    }

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

    pub fn list_type(&self) -> Self {
        match *self {
            Self::BOOLEAN => Self::LIST_BOOLEAN,
            Self::BYTE => Self::LIST_BYTE,
            Self::CHARACTER => Self::LIST_CHARACTER,
            Self::FLOAT => Self::LIST_FLOAT,
            Self::INTEGER => Self::LIST_INTEGER,
            Self::STRING => Self::LIST_STRING,
            Self::MAP => Self::LIST_MAP,
            Self::FUNCTION => Self::LIST_FUNCTION,
            Self::LIST_BOOLEAN
            | Self::LIST_BYTE
            | Self::LIST_CHARACTER
            | Self::LIST_FLOAT
            | Self::LIST_INTEGER
            | Self::LIST_STRING
            | Self::LIST_LIST
            | Self::LIST_MAP
            | Self::LIST_FUNCTION => Self::LIST_LIST,
            _ => *self,
        }
    }

    pub fn destination_type(&self) -> Self {
        match *self {
            Self::CHARACTER_STRING | Self::STRING_CHARACTER => OperandType::STRING,
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
            Self::NONE => write!(f, "none"),
            Self::BOOLEAN => write!(f, "bool"),
            Self::BYTE => write!(f, "byte"),
            Self::CHARACTER => write!(f, "char"),
            Self::FLOAT => write!(f, "float"),
            Self::INTEGER => write!(f, "int"),
            Self::STRING => write!(f, "str"),
            Self::MAP => write!(f, "map"),
            Self::FUNCTION => write!(f, "fn"),
            Self::CHARACTER_STRING => write!(f, "char_str"),
            Self::STRING_CHARACTER => write!(f, "str_char"),
            Self::ARRAY_BOOLEAN => write!(f, "[bool]"),
            Self::ARRAY_BYTE => write!(f, "[byte]"),
            Self::ARRAY_CHARACTER => write!(f, "[char]"),
            Self::ARRAY_FLOAT => write!(f, "[float]"),
            Self::ARRAY_INTEGER => write!(f, "[int]"),
            Self::ARRAY_STRING => write!(f, "[str]"),
            Self::ARRAY_FUNCTION => write!(f, "[fn]"),
            Self::LIST_BOOLEAN => write!(f, "+[bool]"),
            Self::LIST_BYTE => write!(f, "+[byte]"),
            Self::LIST_CHARACTER => write!(f, "+[char]"),
            Self::LIST_FLOAT => write!(f, "+[float]"),
            Self::LIST_INTEGER => write!(f, "+[int]"),
            Self::LIST_STRING => write!(f, "+[str]"),
            Self::LIST_FUNCTION => write!(f, "+[fn]"),
            invalid => write!(f, "INVALID_OPERAND_TYPE({})", invalid.0),
        }
    }
}
