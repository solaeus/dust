use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, Operation};

pub struct Negate {
    pub destination: u16,
    pub operand: Address,
}

impl From<&Instruction> for Negate {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
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
        let a_field = negate.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = negate.operand;

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

impl Display for Negate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Negate {
            destination,
            operand,
        } = self;

        write!(f, "reg_{destination} = -{operand}")
    }
}
