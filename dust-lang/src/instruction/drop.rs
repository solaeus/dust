use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionFields, Operation};

pub struct Drop {
    pub drop_list_start: u16,
    pub drop_list_end: u16,
}

impl From<Instruction> for Drop {
    fn from(instruction: Instruction) -> Self {
        let drop_list_start = instruction.a_field();
        let drop_list_end = instruction.b_field();

        Self {
            drop_list_start,
            drop_list_end,
        }
    }
}

impl From<Drop> for Instruction {
    fn from(drop: Drop) -> Self {
        let operation = Operation::DROP;
        let a_field = drop.drop_list_start;
        let b_field = drop.drop_list_end;

        InstructionFields {
            operation,
            a_field,
            b_field,
            ..Default::default()
        }
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
