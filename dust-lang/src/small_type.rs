/// One-byte representation of a type.
use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

/// One-byte representation of a type. This is an independent and self-contained representation, it is
/// not part of the [`TypeGraph`] and does not have any references to other types.
///
/// For many applications, it is not necessary to have a full representation of a value's type using
/// [`DustType`], which may have heap data and is a rather wasteful way to represent a type if used
/// for every value. It is usually enough just to know how to interpret the value's bits.
///
/// This type is used to encode type information in instructions. To that end, its values must fit
/// in the instruction's D field, which is 6 bits. It's also nice to have a performant way to
/// differentiate heap-allocated types from scalar types. The 5th bit is used to mark heap-allocated
/// types, which allows for a simple check using a bitwise operation while staying within the 6-bit
/// limit.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SmallType(u8);

impl SmallType {
    // Scalar types
    pub const UNIT: SmallType = SmallType(0b0000_0000);
    pub const BOOLEAN: SmallType = SmallType(0b0000_0001);
    pub const BYTE: SmallType = SmallType(0b0000_0010);
    pub const CHARACTER: SmallType = SmallType(0b0000_0011);
    pub const FLOAT: SmallType = SmallType(0b0000_0100);
    pub const INTEGER: SmallType = SmallType(0b0000_0101);
    pub const FUNCTION: SmallType = SmallType(0b0000_0110);
    pub const STRUCT: SmallType = SmallType(0b0000_0111);

    // Heap-allocated types
    // Use the 5th bit to mark heap-allocated types
    pub const STRING: SmallType = SmallType(0b0001_0000);
    pub const LIST_BOOLEAN: SmallType = SmallType(0b0001_0001);
    pub const LIST_BYTE: SmallType = SmallType(0b0001_0010);
    pub const LIST_CHARACTER: SmallType = SmallType(0b0001_0011);
    pub const LIST_FLOAT: SmallType = SmallType(0b0001_0100);
    pub const LIST_INTEGER: SmallType = SmallType(0b0001_0101);
    pub const LIST_STRING: SmallType = SmallType(0b0001_0110);
    pub const LIST_FUNCTION: SmallType = SmallType(0b0001_0111);
    pub const LIST_STRUCT: SmallType = SmallType(0b0001_1000);
    pub const LIST_LIST: SmallType = SmallType(0b0001_1001);
}

impl SmallType {
    pub fn is_scalar(&self) -> bool {
        self.0 & 0b0001_0000 == 0
    }

    pub fn list_type(&self) -> Self {
        match *self {
            Self::BOOLEAN => Self::LIST_BOOLEAN,
            Self::BYTE => Self::LIST_BYTE,
            Self::CHARACTER => Self::LIST_CHARACTER,
            Self::FLOAT => Self::LIST_FLOAT,
            Self::INTEGER => Self::LIST_INTEGER,
            Self::STRING => Self::LIST_STRING,
            Self::FUNCTION => Self::LIST_FUNCTION,
            Self::STRUCT => Self::LIST_STRUCT,
            _ => Self::LIST_LIST,
        }
    }
}

impl Debug for SmallType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for SmallType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::UNIT => write!(f, "none"),
            Self::BOOLEAN => write!(f, "bool"),
            Self::BYTE => write!(f, "byte"),
            Self::CHARACTER => write!(f, "char"),
            Self::FLOAT => write!(f, "float"),
            Self::INTEGER => write!(f, "int"),
            Self::STRING => write!(f, "str"),
            Self::FUNCTION => write!(f, "fn"),
            Self::STRUCT => write!(f, "T"),
            Self::LIST_BOOLEAN => write!(f, "[bool]"),
            Self::LIST_BYTE => write!(f, "[byte]"),
            Self::LIST_CHARACTER => write!(f, "[char]"),
            Self::LIST_FLOAT => write!(f, "[float]"),
            Self::LIST_INTEGER => write!(f, "[int]"),
            Self::LIST_STRING => write!(f, "[str]"),
            Self::LIST_FUNCTION => write!(f, "[fn]"),
            Self::LIST_STRUCT => write!(f, "[T]"),
            Self::LIST_LIST => write!(f, "[[?]]"),
            invalid => write!(f, "<invalid SmallType: {}>", invalid.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_6_bits() {
        const {
            assert!(SmallType::BOOLEAN.0.bit_width() <= 6);
            assert!(SmallType::BYTE.0.bit_width() <= 6);
            assert!(SmallType::CHARACTER.0.bit_width() <= 6);
            assert!(SmallType::FLOAT.0.bit_width() <= 6);
            assert!(SmallType::INTEGER.0.bit_width() <= 6);
            assert!(SmallType::STRING.0.bit_width() <= 6);
            assert!(SmallType::FUNCTION.0.bit_width() <= 6);
            assert!(SmallType::STRUCT.0.bit_width() <= 6);
            assert!(SmallType::LIST_BOOLEAN.0.bit_width() <= 6);
            assert!(SmallType::LIST_BYTE.0.bit_width() <= 6);
            assert!(SmallType::LIST_CHARACTER.0.bit_width() <= 6);
            assert!(SmallType::LIST_FLOAT.0.bit_width() <= 6);
            assert!(SmallType::LIST_INTEGER.0.bit_width() <= 6);
            assert!(SmallType::LIST_STRING.0.bit_width() <= 6);
            assert!(SmallType::LIST_FUNCTION.0.bit_width() <= 6);
            assert!(SmallType::LIST_STRUCT.0.bit_width() <= 6);
            assert!(SmallType::LIST_LIST.0.bit_width() <= 6);
        }
    }

    #[test]
    fn is_scalar() {
        assert!(SmallType::BOOLEAN.is_scalar());
        assert!(SmallType::BYTE.is_scalar());
        assert!(SmallType::CHARACTER.is_scalar());
        assert!(SmallType::FLOAT.is_scalar());
        assert!(SmallType::INTEGER.is_scalar());
        assert!(SmallType::FUNCTION.is_scalar());
        assert!(SmallType::STRUCT.is_scalar());

        assert!(!SmallType::STRING.is_scalar());
        assert!(!SmallType::LIST_BOOLEAN.is_scalar());
        assert!(!SmallType::LIST_BYTE.is_scalar());
        assert!(!SmallType::LIST_CHARACTER.is_scalar());
        assert!(!SmallType::LIST_FLOAT.is_scalar());
        assert!(!SmallType::LIST_INTEGER.is_scalar());
        assert!(!SmallType::LIST_STRING.is_scalar());
        assert!(!SmallType::LIST_FUNCTION.is_scalar());
        assert!(!SmallType::LIST_STRUCT.is_scalar());
        assert!(!SmallType::LIST_LIST.is_scalar());
    }
}
