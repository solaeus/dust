//! A small (4-bit) type representation used to encode the types of instruction operands.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::optimize_inline_capacity;

/// A small (4-bit) type representation used to encode the types of instruction operands.
///
/// `OperandType` represents everything the VM knows about data types. It provides just enough
/// information to determine how to interpret an instruction's operands at runtime. Some built-in
/// types are represented as `OperandType::Pointer`, e.g. `Vec` and `String`. Alebraic types use
/// consecutive registers so struct instances or enum variants can be flattened to
/// [`OperandType::SmallVec`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OperandType(pub(super) u8);

impl OperandType {
    pub type SmallVec = SmallVec<[Self; optimize_inline_capacity::<Self, 0>()]>;

    pub const BOOLEAN: OperandType = Self(0);
    pub const U_8: OperandType = Self(1);
    pub const I_8: OperandType = Self(2);
    pub const U_16: OperandType = Self(3);
    pub const I_16: OperandType = Self(4);
    pub const U_32: OperandType = Self(5);
    pub const I_32: OperandType = Self(6);
    pub const U_64: OperandType = Self(7);
    pub const I_64: OperandType = Self(8);
    pub const U_128: OperandType = Self(9);
    pub const I_128: OperandType = Self(10);
    pub const F_32: OperandType = Self(11);
    pub const F_64: OperandType = Self(12);
    pub const CHARACTER: OperandType = Self(13);
    pub const FUNCTION: OperandType = Self(14);
    pub const POINTER: OperandType = Self(15);

    pub fn register_width(self) -> RegisterWidth {
        match self {
            Self::U_64 | Self::I_64 | Self::F_64 | Self::POINTER => RegisterWidth::Double,
            Self::U_128 | Self::I_128 => RegisterWidth::Quad,
            _ => RegisterWidth::Single,
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
            Self::POINTER => write!(f, "ptr"),
            _ => write!(f, "<invalid operand type>"),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum RegisterWidth {
    Single,
    Double,
    Quad,
}

impl RegisterWidth {
    pub fn as_u16(&self) -> u16 {
        match self {
            RegisterWidth::Single => 1,
            RegisterWidth::Double => 2,
            RegisterWidth::Quad => 4,
        }
    }

    pub fn as_usize(&self) -> usize {
        match self {
            RegisterWidth::Single => 1,
            RegisterWidth::Double => 2,
            RegisterWidth::Quad => 4,
        }
    }
}
