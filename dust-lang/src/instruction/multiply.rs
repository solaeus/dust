use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, Operation};

pub struct Multiply {
    pub destination: u16,
    pub left: Address,
    pub right: Address,
}

impl From<&Instruction> for Multiply {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let left = instruction.b_address();
        let right = instruction.c_address();

        Multiply {
            destination,
            left,
            right,
        }
    }
}

impl From<Multiply> for Instruction {
    fn from(multiply: Multiply) -> Self {
        let operation = Operation::MULTIPLY;
        let a_field = multiply.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = multiply.left;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = multiply.right;

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

impl Display for Multiply {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Multiply {
            destination,
            left,
            right,
        } = self;

        write!(f, "reg_{destination} = {left} × {right}")
    }
}
