use std::fmt::{self, Display, Formatter};

use super::{Address, Destination, Instruction, InstructionFields, Operation};

pub struct Multiply {
    pub destination: Destination,
    pub left: Address,
    pub right: Address,
}

impl From<Instruction> for Multiply {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
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
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = multiply.destination;
        let Address {
            index: b_field,
            kind: b_kind,
        } = multiply.left;
        let Address {
            index: c_field,
            kind: c_kind,
        } = multiply.right;

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

impl Display for Multiply {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Multiply {
            destination,
            left,
            right,
        } = self;

        destination.display(f, left.r#type())?;
        write!(f, " = {left} ✕ {right}",)
    }
}
