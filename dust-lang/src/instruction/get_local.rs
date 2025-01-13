use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::InstructionBuilder;

pub struct GetLocal {
    pub destination: u16,
    pub local_index: u16,
}

impl From<Instruction> for GetLocal {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let local_index = instruction.b_field();

        GetLocal {
            destination,
            local_index,
        }
    }
}

impl From<GetLocal> for Instruction {
    fn from(get_local: GetLocal) -> Self {
        let operation = Operation::GET_LOCAL;
        let a_field = get_local.destination;
        let b_field = get_local.local_index;

        InstructionBuilder {
            operation,
            a_field,
            b_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for GetLocal {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let GetLocal {
            destination,
            local_index,
        } = self;

        write!(f, "R{destination} = L{local_index}")
    }
}
