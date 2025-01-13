use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::InstructionBuilder;

pub struct LoadList {
    pub destination: u16,
    pub start_register: u16,
    pub jump_next: bool,
}

impl From<Instruction> for LoadList {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let start_register = instruction.b_field();
        let jump_next = instruction.c_field() != 0;

        LoadList {
            destination,
            start_register,
            jump_next,
        }
    }
}

impl From<LoadList> for Instruction {
    fn from(load_list: LoadList) -> Self {
        InstructionBuilder {
            operation: Operation::LOAD_LIST,
            a_field: load_list.destination,
            b_field: load_list.start_register,
            c_field: load_list.jump_next as u16,
            ..Default::default()
        }
        .build()
    }
}

impl Display for LoadList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LoadList {
            destination,
            start_register,
            jump_next,
        } = self;

        write!(f, "R{destination} = [R{start_register}..R{destination}]")?;

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
