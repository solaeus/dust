use std::fmt::{self, Display, Formatter};

use super::{Address, Destination, Instruction, InstructionFields, Operation};

pub struct Modulo {
    pub destination: Destination,
    pub left: Address,
    pub right: Address,
}

impl From<Instruction> for Modulo {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
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
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = modulo.destination;
        let Address {
            index: b_field,
            kind: b_kind,
        } = modulo.left;
        let Address {
            index: c_field,
            kind: c_kind,
        } = modulo.right;

        InstructionFields {
            operation,
            a_field,
            a_is_register,
            b_field,
            b_kind,
            c_field,
            c_kind,
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

        destination.display(f, left.r#type())?;
        write!(f, " = {left} % {right}")
    }
}
