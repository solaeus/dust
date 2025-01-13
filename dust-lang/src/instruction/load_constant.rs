use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::InstructionBuilder;

pub struct LoadConstant {
    pub destination: u16,
    pub constant_index: u16,
    pub jump_next: bool,
}

impl From<Instruction> for LoadConstant {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let constant_index = instruction.b_field();
        let jump_next = instruction.c_field() != 0;

        LoadConstant {
            destination,
            constant_index,
            jump_next,
        }
    }
}

impl From<LoadConstant> for Instruction {
    fn from(load_constant: LoadConstant) -> Self {
        InstructionBuilder {
            operation: Operation::LOAD_CONSTANT,
            a_field: load_constant.destination,
            b_field: load_constant.constant_index,
            c_field: load_constant.jump_next as u16,
            ..Default::default()
        }
        .build()
    }
}

impl Display for LoadConstant {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LoadConstant {
            destination,
            constant_index,
            jump_next,
        } = self;

        write!(f, "R{destination} = C{constant_index}")?;

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
