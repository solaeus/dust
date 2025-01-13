use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operation};

pub struct LoadFunction {
    pub destination: u16,
    pub prototype_index: u16,
    pub jump_next: bool,
}

impl From<Instruction> for LoadFunction {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let prototype_index = instruction.b_field();
        let jump_next = instruction.c_field() != 0;

        LoadFunction {
            destination,
            prototype_index,
            jump_next,
        }
    }
}

impl From<LoadFunction> for Instruction {
    fn from(load_function: LoadFunction) -> Self {
        InstructionBuilder {
            operation: Operation::LOAD_FUNCTION,
            a_field: load_function.destination,
            b_field: load_function.prototype_index,
            c_field: load_function.jump_next as u16,
            ..Default::default()
        }
        .build()
    }
}

impl Display for LoadFunction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LoadFunction {
            destination,
            prototype_index,
            jump_next,
        } = self;

        write!(f, "R{destination} = P{prototype_index}")?;

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
