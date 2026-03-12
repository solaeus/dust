use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

/// A small (4-bit) type representation used to encode the types of instruction operands.
///
/// `OperandType` can represent any type, but not always with full specificity. It provides just
/// enough information to determine how to interpret an instruction's operands at runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OperandType(pub(super) u8);

impl OperandType {
    pub const BOOLEAN: OperandType = OperandType(0);
    pub const U_8: OperandType = OperandType(1);
    pub const I_8: OperandType = OperandType(2);
    pub const U_16: OperandType = OperandType(3);
    pub const I_16: OperandType = OperandType(4);
    pub const U_32: OperandType = OperandType(5);
    pub const I_32: OperandType = OperandType(6);
    pub const U_64: OperandType = OperandType(7);
    pub const I_64: OperandType = OperandType(8);
    pub const U_128: OperandType = OperandType(9);
    pub const I_128: OperandType = OperandType(10);
    pub const F_32: OperandType = OperandType(11);
    pub const F_64: OperandType = OperandType(12);
    pub const CHARACTER: OperandType = OperandType(13);
    pub const FUNCTION: OperandType = OperandType(14);
    pub const POINTER: OperandType = OperandType(15);

    /// Returns the byte size of values of this type or `None`
    pub fn size_in_bytes(self) -> usize {
        match self {
            Self::BOOLEAN | Self::CHARACTER | Self::U_8 | Self::I_8 => 1,
            Self::U_16 | Self::I_16 | Self::FUNCTION => 2,
            Self::U_32 | Self::I_32 | Self::F_32 => 4,
            Self::U_64 | Self::I_64 | Self::F_64 => 8,
            Self::U_128 | Self::I_128 => 16,
            Self::POINTER => size_of::<usize>(),
            _ => panic!("Invalid operand type: {self:?}"),
        }
    }
}

impl Display for OperandType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Self::BOOLEAN => write!(f, "bool"),
            Self::U_8 => write!(f, "u8"),
            Self::I_8 => write!(f, "i8"),
            Self::U_16 => write!(f, "u16"),
            Self::I_16 => write!(f, "i16"),
            Self::U_32 => write!(f, "u32"),
            Self::I_32 => write!(f, "i32"),
            Self::U_64 => write!(f, "u64"),
            Self::I_64 => write!(f, "i64"),
            Self::U_128 => write!(f, "u128"),
            Self::I_128 => write!(f, "i128"),
            Self::F_32 => write!(f, "f32"),
            Self::F_64 => write!(f, "f64"),
            Self::CHARACTER => write!(f, "char"),
            Self::FUNCTION => write!(f, "fn"),
            Self::POINTER => write!(f, "*T"),
            _ => write!(f, "<invalid operand type>"),
        }
    }
}
