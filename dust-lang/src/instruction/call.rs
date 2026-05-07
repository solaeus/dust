use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionBuilder, Operation};

pub struct Call {
    pub destination: u16,
    pub callee: Address,
    pub arguments_start: u16,
}

impl From<Instruction> for Call {
    fn from(instruction: Instruction) -> Self {
        Call {
            destination: instruction.a_field(),
            callee: instruction.b_address(),
            arguments_start: instruction.c_field(),
        }
    }
}

impl From<Call> for Instruction {
    fn from(call: Call) -> Self {
        let Call {
            destination,
            callee,
            arguments_start,
        } = call;

        InstructionBuilder::new(Operation::CALL)
            .a_field(destination)
            .b_address(callee)
            .c_field(arguments_start)
            .build()
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Call {
            destination,
            callee,
            arguments_start,
        } = *self;

        if destination != u16::MAX {
            write!(f, "reg_{destination} = ")?;
        }

        write!(f, "{callee}")?;

        if arguments_start == u16::MAX {
            write!(f, "()")
        } else {
            write!(f, "(reg_{arguments_start}...)")
        }
    }
}
