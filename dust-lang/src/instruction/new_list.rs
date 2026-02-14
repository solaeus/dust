use std::fmt::{self, Display, Formatter};

use crate::instruction::Address;

use super::{Instruction, InstructionFields, Operation};

pub struct NewList {
    pub destination: u16,
    pub initial_length: Address,
}

impl From<&Instruction> for NewList {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let initial_length = instruction.b_address();

        NewList {
            destination,
            initial_length,
        }
    }
}

impl From<NewList> for Instruction {
    fn from(list: NewList) -> Self {
        let operation = Operation::NEW_LIST;
        let a_field = list.destination;
        let b_field = list.initial_length.index;
        let b_memory_kind = list.initial_length.memory;

        InstructionFields {
            operation,
            a_field,
            b_field,
            b_memory_kind,
            ..Default::default()
        }
        .build()
    }
}

impl Display for NewList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let NewList {
            destination,
            initial_length,
        } = self;

        write!(f, "reg_{destination} = [; {initial_length}]")
    }
}
