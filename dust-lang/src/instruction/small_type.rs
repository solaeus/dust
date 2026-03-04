/// 6-bit representation of a type.
use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

/// 6-bit representation of a type. This is an independent and self-contained representation, it is
/// not part of the [`TypeGraph`] and does not have any references to other types.
///
/// For many applications, it is not necessary to have a full representation of a value's type using
/// [`DustType`], which may have heap data and is a rather wasteful way to represent a type if used
/// for every value. It is usually enough just to know how to interpret the value's bits.
///
/// `SmallType` is also used to encode type information in instructions. To that end, its values
/// must fit in the instruction's D field, which is 6 bits. It's also nice to have a performant way
/// to differentiate heap-allocated types from scalar types. The 6th bit is used as a marker, which
/// allows for a simple check using a bitwise operation while staying within the 6-bit limit.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SmallType(pub(super) u8);

impl SmallType {
    // Scalar types
    pub const UNIT: SmallType = SmallType(0b0000_0000);
    pub const BOOLEAN: SmallType = SmallType(0b0000_0001);
    pub const CHARACTER: SmallType = SmallType(0b0000_0010);
    pub const U_8: SmallType = SmallType(0b0000_0011);
    pub const I_8: SmallType = SmallType(0b0000_0100);
    pub const U_16: SmallType = SmallType(0b0000_0101);
    pub const I_16: SmallType = SmallType(0b0000_0110);
    pub const U_32: SmallType = SmallType(0b0000_0111);
    pub const I_32: SmallType = SmallType(0b0000_1000);
    pub const U_64: SmallType = SmallType(0b0000_1001);
    pub const I_64: SmallType = SmallType(0b0000_1010);
    pub const U_128: SmallType = SmallType(0b0000_1011);
    pub const I_128: SmallType = SmallType(0b0000_1100);
    pub const F_32: SmallType = SmallType(0b0000_1101);
    pub const F_64: SmallType = SmallType(0b0000_1110);
    pub const FUNCTION: SmallType = SmallType(0b0000_1111);
    pub const STRUCT: SmallType = SmallType(0b0001_0000);

    // Heap-allocated types
    // Use the 6th bit to mark heap-allocated types
    pub const STRING: SmallType = SmallType(0b0010_0000);
    pub const LIST_BOOLEAN: SmallType = SmallType(0b0010_0001);
    pub const LIST_CHARACTER: SmallType = SmallType(0b0010_0010);
    pub const LIST_U_8: SmallType = SmallType(0b0010_0011);
    pub const LIST_I_8: SmallType = SmallType(0b0010_0100);
    pub const LIST_U_16: SmallType = SmallType(0b0010_0101);
    pub const LIST_I_16: SmallType = SmallType(0b0010_0110);
    pub const LIST_U_32: SmallType = SmallType(0b0010_0111);
    pub const LIST_I_32: SmallType = SmallType(0b0010_1000);
    pub const LIST_U_64: SmallType = SmallType(0b0010_1001);
    pub const LIST_I_64: SmallType = SmallType(0b0010_1010);
    pub const LIST_U_128: SmallType = SmallType(0b0010_1011);
    pub const LIST_I_128: SmallType = SmallType(0b0010_1100);
    pub const LIST_F_32: SmallType = SmallType(0b0010_1101);
    pub const LIST_F_64: SmallType = SmallType(0b0010_1110);
    pub const LIST_FUNCTION: SmallType = SmallType(0b0010_1111);
    pub const LIST_STRUCT: SmallType = SmallType(0b0011_0000);
    pub const LIST_STRING: SmallType = SmallType(0b0011_0001);
    pub const LIST_LIST: SmallType = SmallType(0b0011_0010);
}

impl SmallType {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::UNIT => "none",
            Self::BOOLEAN => "bool",
            Self::CHARACTER => "char",
            Self::STRING => "str",
            Self::U_8 => "u8",
            Self::I_8 => "i8",
            Self::U_16 => "u16",
            Self::I_16 => "i16",
            Self::U_32 => "u32",
            Self::I_32 => "i32",
            Self::U_64 => "u64",
            Self::I_64 => "i64",
            Self::U_128 => "u128",
            Self::I_128 => "i128",
            Self::F_32 => "f32",
            Self::F_64 => "f64",
            Self::FUNCTION => "fn",
            Self::STRUCT => "T",
            Self::LIST_BOOLEAN => "[bool]",
            Self::LIST_CHARACTER => "[char]",
            Self::LIST_STRING => "[str]",
            Self::LIST_U_8 => "[u8]",
            Self::LIST_I_8 => "[i8]",
            Self::LIST_U_16 => "[u16]",
            Self::LIST_I_16 => "[i16]",
            Self::LIST_U_32 => "[u32]",
            Self::LIST_I_32 => "[i32]",
            Self::LIST_U_64 => "[u64]",
            Self::LIST_I_64 => "[i64]",
            Self::LIST_U_128 => "[u128]",
            Self::LIST_I_128 => "[i128]",
            Self::LIST_F_32 => "[f32]",
            Self::LIST_F_64 => "[f64]",
            Self::LIST_FUNCTION => "[fn]",
            Self::LIST_STRUCT => "[T]",
            Self::LIST_LIST => "[[T]]",
            _ => "<invalid SmallType>",
        }
    }

    pub fn is_scalar(&self) -> bool {
        self.0 & 0b0010_0000 == 0
    }

    pub fn list_type(&self) -> Self {
        match *self {
            Self::BOOLEAN => Self::LIST_BOOLEAN,
            Self::CHARACTER => Self::LIST_CHARACTER,
            Self::U_8 => Self::LIST_U_8,
            Self::I_8 => Self::LIST_I_8,
            Self::U_16 => Self::LIST_U_16,
            Self::I_16 => Self::LIST_I_16,
            Self::U_32 => Self::LIST_U_32,
            Self::I_32 => Self::LIST_I_32,
            Self::U_64 => Self::LIST_U_64,
            Self::I_64 => Self::LIST_I_64,
            Self::U_128 => Self::LIST_U_128,
            Self::I_128 => Self::LIST_I_128,
            Self::F_32 => Self::LIST_F_32,
            Self::F_64 => Self::LIST_F_64,
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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_6_bits() {
        const {
            assert!(SmallType::BOOLEAN.0.bit_width() <= 6);
            assert!(SmallType::CHARACTER.0.bit_width() <= 6);
            assert!(SmallType::F_32.0.bit_width() <= 6);
            assert!(SmallType::F_64.0.bit_width() <= 6);
            assert!(SmallType::U_8.0.bit_width() <= 6);
            assert!(SmallType::I_8.0.bit_width() <= 6);
            assert!(SmallType::U_16.0.bit_width() <= 6);
            assert!(SmallType::I_16.0.bit_width() <= 6);
            assert!(SmallType::U_32.0.bit_width() <= 6);
            assert!(SmallType::I_32.0.bit_width() <= 6);
            assert!(SmallType::U_64.0.bit_width() <= 6);
            assert!(SmallType::I_64.0.bit_width() <= 6);
            assert!(SmallType::U_128.0.bit_width() <= 6);
            assert!(SmallType::I_128.0.bit_width() <= 6);
            assert!(SmallType::STRING.0.bit_width() <= 6);
            assert!(SmallType::FUNCTION.0.bit_width() <= 6);
            assert!(SmallType::STRUCT.0.bit_width() <= 6);
            assert!(SmallType::LIST_BOOLEAN.0.bit_width() <= 6);
            assert!(SmallType::LIST_CHARACTER.0.bit_width() <= 6);
            assert!(SmallType::LIST_U_8.0.bit_width() <= 6);
            assert!(SmallType::LIST_I_8.0.bit_width() <= 6);
            assert!(SmallType::LIST_U_16.0.bit_width() <= 6);
            assert!(SmallType::LIST_I_16.0.bit_width() <= 6);
            assert!(SmallType::LIST_U_32.0.bit_width() <= 6);
            assert!(SmallType::LIST_I_32.0.bit_width() <= 6);
            assert!(SmallType::LIST_U_64.0.bit_width() <= 6);
            assert!(SmallType::LIST_I_64.0.bit_width() <= 6);
            assert!(SmallType::LIST_U_128.0.bit_width() <= 6);
            assert!(SmallType::LIST_I_128.0.bit_width() <= 6);
            assert!(SmallType::LIST_F_32.0.bit_width() <= 6);
            assert!(SmallType::LIST_F_64.0.bit_width() <= 6);
            assert!(SmallType::LIST_FUNCTION.0.bit_width() <= 6);
            assert!(SmallType::LIST_STRUCT.0.bit_width() <= 6);
            assert!(SmallType::LIST_STRING.0.bit_width() <= 6);
            assert!(SmallType::LIST_LIST.0.bit_width() <= 6);
        }
    }

    #[test]
    fn is_scalar() {
        assert!(SmallType::BOOLEAN.is_scalar());
        assert!(SmallType::CHARACTER.is_scalar());
        assert!(SmallType::F_32.is_scalar());
        assert!(SmallType::F_64.is_scalar());
        assert!(SmallType::U_8.is_scalar());
        assert!(SmallType::I_8.is_scalar());
        assert!(SmallType::U_16.is_scalar());
        assert!(SmallType::I_16.is_scalar());
        assert!(SmallType::U_32.is_scalar());
        assert!(SmallType::I_32.is_scalar());
        assert!(SmallType::U_64.is_scalar());
        assert!(SmallType::I_64.is_scalar());
        assert!(SmallType::U_128.is_scalar());
        assert!(SmallType::I_128.is_scalar());
        assert!(SmallType::FUNCTION.is_scalar());
        assert!(SmallType::STRUCT.is_scalar());

        assert!(!SmallType::STRING.is_scalar());
        assert!(!SmallType::LIST_BOOLEAN.is_scalar());
        assert!(!SmallType::LIST_CHARACTER.is_scalar());
        assert!(!SmallType::LIST_U_8.is_scalar());
        assert!(!SmallType::LIST_I_8.is_scalar());
        assert!(!SmallType::LIST_U_16.is_scalar());
        assert!(!SmallType::LIST_I_16.is_scalar());
        assert!(!SmallType::LIST_U_32.is_scalar());
        assert!(!SmallType::LIST_I_32.is_scalar());
        assert!(!SmallType::LIST_U_64.is_scalar());
        assert!(!SmallType::LIST_I_64.is_scalar());
        assert!(!SmallType::LIST_U_128.is_scalar());
        assert!(!SmallType::LIST_I_128.is_scalar());
        assert!(!SmallType::LIST_F_32.is_scalar());
        assert!(!SmallType::LIST_F_64.is_scalar());
        assert!(!SmallType::LIST_FUNCTION.is_scalar());
        assert!(!SmallType::LIST_STRUCT.is_scalar());
        assert!(!SmallType::LIST_STRING.is_scalar());
        assert!(!SmallType::LIST_LIST.is_scalar());
    }
}
