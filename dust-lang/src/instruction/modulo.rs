use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, Operation};

pub struct Modulo {
    pub destination: u16,
    pub left: Address,
    pub right: Address,
}

impl From<&Instruction> for Modulo {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let left = instruction.b_address();
        let right = instruction.c_address();

        Modulo {
            destination,
            left,
            right,
        }
    }
}

impl From<Modulo> for Instruction {
    fn from(modulo: Modulo) -> Self {
        let operation = Operation::MODULO;
        let a_field = modulo.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = modulo.left;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = modulo.right;

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

impl Display for Modulo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Modulo {
            destination,
            left,
            right,
        } = self;

        write!(f, "reg_{destination} = {left} % {right}")
    }
}
