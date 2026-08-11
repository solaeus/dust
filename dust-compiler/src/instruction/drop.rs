use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, Operation};

pub struct Drop {
    pub start_register: u16,
    pub end_register: u16,
}

impl From<Instruction> for Drop {
    fn from(instruction: Instruction) -> Self {
        Self {
            start_register: instruction.a_field() as u16,
            end_register: instruction.b_field() as u16,
        }
    }
}

impl From<Drop> for Instruction {
    fn from(drop: Drop) -> Self {
        let Drop {
            start_register,
            end_register,
        } = drop;

        InstructionBuilder::new(Operation::DROP)
            .a_field(start_register)
            .b_field(end_register)
            .build()
    }
}

impl Display for Drop {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Drop {
            start_register,
            end_register,
        } = self;

        write!(f, "drop {start_register}..{end_register}")
    }
}
