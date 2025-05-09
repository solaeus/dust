use std::fmt::{self, Display, Formatter};

use super::{Address, Destination, Instruction, InstructionFields, Operation};

pub struct Divide {
    pub destination: Destination,
    pub left: Address,
    pub right: Address,
}

impl From<Instruction> for Divide {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
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
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = divide.destination;
        let Address {
            index: b_field,
            kind: b_kind,
        } = divide.left;
        let Address {
            index: c_field,
            kind: c_kind,
        } = divide.right;

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

impl Display for Divide {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Divide {
            destination,
            left,
            right,
        } = self;

        destination.display(f, left.r#type())?;
        write!(f, " = {left} ÷ {right}")
    }
}
