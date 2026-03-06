use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::register::RegisterClass;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct OperandType(pub(super) u8);

impl OperandType {
    // RegisterClass::Integer32
    pub const BOOLEAN: OperandType = OperandType(0);
    pub const CHARACTER: OperandType = OperandType(1);
    pub const U_8: OperandType = OperandType(2);
    pub const I_8: OperandType = OperandType(3);
    pub const U_16: OperandType = OperandType(4);
    pub const I_16: OperandType = OperandType(5);
    pub const U_32: OperandType = OperandType(6);
    pub const I_32: OperandType = OperandType(7);
    pub const FUNCTION: OperandType = OperandType(14);

    // RegisterClass::Integer64
    pub const U_64: OperandType = OperandType(8);
    pub const I_64: OperandType = OperandType(9);
    pub const U_128: OperandType = OperandType(10);
    pub const I_128: OperandType = OperandType(11);

    // RegisterClass::Float64
    pub const F_32: OperandType = OperandType(12);
    pub const F_64: OperandType = OperandType(13);

    // RegisterClass::Pointer
    pub const STRING: OperandType = OperandType(15);
    pub const LIST: OperandType = OperandType(16);

    // Used for ADD instructions that concatenate strings with characters.
    pub const CHARACTER_STRING: OperandType = OperandType(17);
    pub const STRING_CHARACTER: OperandType = OperandType(18);

    pub fn register_class(self) -> RegisterClass {
        match self {
            Self::BOOLEAN
            | Self::CHARACTER
            | Self::U_8
            | Self::I_8
            | Self::U_16
            | Self::I_16
            | Self::U_32
            | Self::I_32
            | Self::FUNCTION => RegisterClass::Integer32,
            Self::U_64 | Self::I_64 | Self::U_128 | Self::I_128 => RegisterClass::Integer64,
            Self::F_32 | Self::F_64 => RegisterClass::Float64,
            _ => RegisterClass::Pointer,
        }
    }
}

impl Display for OperandType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Self::BOOLEAN => write!(f, "bool"),
            Self::CHARACTER => write!(f, "char"),
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
            Self::FUNCTION => write!(f, "fn"),
            Self::STRING => write!(f, "str"),
            Self::LIST => write!(f, "[]"),
            _ => write!(f, "<invalid operand type>"),
        }
    }
}
