use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, Operation};

pub struct Divide {
    pub destination: u16,
    pub left: Address,
    pub right: Address,
}

impl From<&Instruction> for Divide {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let left = instruction.b_address();
        let right = instruction.c_address();

        Divide {
            destination,
            left,
            right,
        }
    }
}

impl From<Divide> for Instruction {
    fn from(divide: Divide) -> Self {
        let operation = Operation::DIVIDE;
        let a_field = divide.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = divide.left;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = divide.right;

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

impl Display for Divide {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Divide {
            destination,
            left,
            right,
        } = self;

        write!(f, "reg_{destination} = {left} ÷ {right}")
    }
}
