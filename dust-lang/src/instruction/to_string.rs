use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionFields, Operation};

pub struct ToString {
    pub destination: u16,
    pub operand: Address,
}

impl From<&Instruction> for ToString {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let operand = Address {
            index: instruction.b_field(),
            memory: instruction.b_memory_kind(),
        };

        ToString {
            destination,
            operand,
        }
    }
}

impl From<ToString> for Instruction {
    fn from(to_string: ToString) -> Self {
        let operation = Operation::TO_STRING;
        let a_field = to_string.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = to_string.operand;

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

impl Display for ToString {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let ToString {
            destination,
            operand,
        } = self;

        write!(f, "reg_{destination} = {operand} as str")
    }
}
