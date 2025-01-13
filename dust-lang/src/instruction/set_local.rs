use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::InstructionBuilder;

pub struct SetLocal {
    pub register_index: u16,
    pub local_index: u16,
}

impl From<Instruction> for SetLocal {
    fn from(instruction: Instruction) -> Self {
        let register_index = instruction.b_field();
        let local_index = instruction.c_field();

        SetLocal {
            register_index,
            local_index,
        }
    }
}

impl From<SetLocal> for Instruction {
    fn from(set_local: SetLocal) -> Self {
        let operation = Operation::SET_LOCAL;
        let b_field = set_local.register_index;
        let c_field = set_local.local_index;

        InstructionBuilder {
            operation,
            b_field,
            c_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for SetLocal {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let SetLocal {
            register_index,
            local_index,
        } = self;

        write!(f, "L{local_index} = R{register_index}")
    }
}
