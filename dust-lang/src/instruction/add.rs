use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, Operation};

pub struct Add {
    pub destination: u16,
    pub left: Address,
    pub right: Address,
}

impl From<&Instruction> for Add {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let left = instruction.b_address();
        let right = instruction.c_address();

        Add {
            destination,
            left,
            right,
        }
    }
}

impl From<Add> for Instruction {
    fn from(add: Add) -> Self {
        let operation = Operation::ADD;
        let a_field = add.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = add.left;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = add.right;

        InstructionFields {
            operation,
            a_field,
            b_field,
            b_memory_kind,
            c_field,
            c_memory_kind,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Add {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Add {
            destination,
            left,
            right,
        } = self;

        write!(f, "reg_{destination} = {left} + {right}")
    }
}
