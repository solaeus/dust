use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::InstructionBuilder;

pub struct LoadSelf {
    pub destination: u16,
    pub jump_next: bool,
}

impl From<Instruction> for LoadSelf {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let jump_next = instruction.c_field() != 0;

        LoadSelf {
            destination,
            jump_next,
        }
    }
}

impl From<LoadSelf> for Instruction {
    fn from(load_self: LoadSelf) -> Self {
        InstructionBuilder {
            operation: Operation::LOAD_SELF,
            a_field: load_self.destination,
            c_field: load_self.jump_next as u16,
            ..Default::default()
        }
        .build()
    }
}

impl Display for LoadSelf {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LoadSelf {
            destination,
            jump_next,
        } = self;

        write!(f, "R{destination} = SELF")?;

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
