use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, Operation};

pub struct Subtract {
    pub destination: u16,
    pub left: Address,
    pub right: Address,
}

impl From<&Instruction> for Subtract {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let left = instruction.b_address();
        let right = instruction.c_address();

        Subtract {
            destination,
            left,
            right,
        }
    }
}

impl From<Subtract> for Instruction {
    fn from(subtract: Subtract) -> Self {
        let operation = Operation::SUBTRACT;
        let a_field = subtract.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = subtract.left;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = subtract.right;

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

impl Display for Subtract {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Subtract {
            destination,
            left,
            right,
        } = self;

        write!(f, "reg_{destination} = {left} - {right}")
    }
}
