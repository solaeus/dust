use std::fmt::{self, Display, Formatter};

use crate::{Instruction, InstructionFields};

pub struct Drop {
    pub drop_list_index: u16,
}

impl From<Instruction> for Drop {
    fn from(instruction: Instruction) -> Self {
        let drop_list_index = instruction.a_field();

        Self { drop_list_index }
    }
}

impl From<Drop> for Instruction {
    fn from(safepoint: Drop) -> Self {
        let operation = crate::Operation::DROP;
        let a_field = safepoint.drop_list_index;

        InstructionFields {
            operation,
            a_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Drop {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.drop_list_index == u16::MAX {
            Ok(())
        } else {
            write!(f, "drop_list_{}", self.drop_list_index)
        }
    }
}
