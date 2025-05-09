use std::fmt::{self, Display, Formatter};

use super::{Address, Destination, Instruction, InstructionFields, Operation};

pub struct Negate {
    pub destination: Destination,
    pub operand: Address,
}

impl From<Instruction> for Negate {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let operand = instruction.b_address();

        Negate {
            destination,
            operand,
        }
    }
}

impl From<Negate> for Instruction {
    fn from(negate: Negate) -> Self {
        let operation = Operation::NEGATE;
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = negate.destination;
        let Address {
            index: b_field,
            kind: b_kind,
        } = negate.operand;

        InstructionFields {
            operation,
            a_field,
            a_is_register,
            b_field,
            b_kind,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Negate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Negate {
            destination,
            operand,
        } = self;

        destination.display(f, operand.r#type());
        write!(f, " = -{operand}",)
    }
}
