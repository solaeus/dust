use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct InstructionOptions: u8 {
        const A_IS_LOCAL = 0b00000001;

        const B_IS_CONSTANT = 0b00000010;
        const B_IS_LOCAL = 0b00000100;

        const C_IS_CONSTANT = 0b00001000;
        const C_IS_LOCAL = 0b00010000;

        const D_IS_TRUE = 0b00100000;
    }
}

impl InstructionOptions {
    pub fn a_is_local(self) -> bool {
        self.contains(Self::A_IS_LOCAL)
    }

    pub fn b_is_constant(self) -> bool {
        self.contains(Self::B_IS_CONSTANT)
    }

    pub fn b_is_local(self) -> bool {
        self.contains(Self::B_IS_LOCAL)
    }

    pub fn c_is_constant(self) -> bool {
        self.contains(Self::C_IS_CONSTANT)
    }

    pub fn c_is_local(self) -> bool {
        self.contains(Self::C_IS_LOCAL)
    }

    pub fn d(self) -> bool {
        self.contains(Self::D_IS_TRUE)
    }
}
