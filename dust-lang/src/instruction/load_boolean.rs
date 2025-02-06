use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::InstructionFields;

pub struct LoadBoolean {
    pub destination: u16,
    pub value: bool,
    pub jump_next: bool,
}

impl From<Instruction> for LoadBoolean {
    fn from(instruction: Instruction) -> Self {
        LoadBoolean {
            destination: instruction.a_field(),
            value: instruction.b_field() != 0,
            jump_next: instruction.c_field() != 0,
        }
    }
}

impl From<LoadBoolean> for Instruction {
    fn from(load_boolean: LoadBoolean) -> Self {
        let operation = Operation::LOAD_BOOLEAN;
        let a_field = load_boolean.destination;
        let b_field = load_boolean.value as u16;
        let c_field = load_boolean.jump_next as u16;

        InstructionFields {
            operation,
            a_field,
            b_field,
            c_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for LoadBoolean {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LoadBoolean {
            destination,
            value,
            jump_next,
        } = self;

        write!(f, "R{destination} = {value}")?;

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
