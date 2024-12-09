//! Byte that uses its bits as boolean flags to represent information about an instruction's
//! arguments. Additionally, one bit is used as the instruction's `D` field.
//!
//! See the [instruction documentation](crate::instruction) for more information.
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    /// Byte that uses its bits as boolean flags to represent an instruction's options and D field.
    ///
    /// See the [instruction documentation](crate::instruction) for more information.
    #[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct InstructionOptions: u8 {
        const B_IS_CONSTANT = 0b0010_0000;

        const C_IS_CONSTANT = 0b0100_0000;

        const D_IS_TRUE = 0b1000_0000;
    }
}

impl InstructionOptions {
    pub fn b_is_constant(self) -> bool {
        self.contains(Self::B_IS_CONSTANT)
    }

    pub fn c_is_constant(self) -> bool {
        self.contains(Self::C_IS_CONSTANT)
    }

    pub fn d(self) -> bool {
        self.contains(Self::D_IS_TRUE)
    }
}
