use std::fmt::{self, Display, Formatter};

use crate::{Instruction, InstructionFields};

pub struct Safepoint {
    pub safepoint_index: u16,
}

impl From<Instruction> for Safepoint {
    fn from(instruction: Instruction) -> Self {
        let safepoint_index = instruction.a_field();

        Self { safepoint_index }
    }
}

impl From<Safepoint> for Instruction {
    fn from(safepoint: Safepoint) -> Self {
        let operation = crate::Operation::SAFEPOINT;
        let a_field = safepoint.safepoint_index;

        InstructionFields {
            operation,
            a_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Safepoint {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "sp_{}", self.safepoint_index)
    }
}
