use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, Operation};

pub struct Drop {
    pub drop_list_start: u16,
    pub drop_list_end: u16,
}

impl From<&Instruction> for Drop {
    fn from(instruction: &Instruction) -> Self {
        Self {
            drop_list_start: instruction.a_field(),
            drop_list_end: instruction.b_field(),
        }
    }
}

impl From<Drop> for Instruction {
    fn from(drop: Drop) -> Self {
        let Drop {
            drop_list_start,
            drop_list_end,
        } = drop;

        InstructionBuilder::new(Operation::DROP)
            .a_field(drop_list_start)
            .b_field(drop_list_end)
            .build()
    }
}

impl Display for Drop {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Drop {
            drop_list_start,
            drop_list_end,
        } = self;

        write!(f, "drop {drop_list_start}..{drop_list_end}")
    }
}
