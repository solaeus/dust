use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, MemoryKind, Operation};

pub struct Call {
    pub destination: u16,
    pub callee_memory: MemoryKind,
    pub callee_index: u16,
    pub arguments_start: u16,
    pub argument_count: u16,
}

impl From<&Instruction> for Call {
    fn from(instruction: &Instruction) -> Self {
        Call {
            destination: instruction.a_field(),
            callee_memory: instruction.b_memory(),
            callee_index: instruction.b_field(),
            arguments_start: instruction.c_field(),
            argument_count: instruction.d_field(),
        }
    }
}

impl From<Call> for Instruction {
    fn from(call: Call) -> Self {
        let Call {
            destination,
            callee_memory,
            callee_index,
            arguments_start,
            argument_count,
        } = call;

        InstructionBuilder::new(Operation::CALL)
            .a_field(destination)
            .b_memory(callee_memory)
            .b_field(callee_index)
            .c_field(arguments_start)
            .d_field(argument_count)
            .build()
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Call {
            destination,
            callee_memory,
            callee_index,
            arguments_start,
            argument_count,
        } = *self;

        if destination != u16::MAX {
            write!(f, "reg_{destination} = ")?;
        }

        write!(f, "{callee_memory}_{callee_index}")?;

        if argument_count == 0 {
            write!(f, "()")
        } else {
            let arguments_end = arguments_start + argument_count;

            write!(f, "(args_{arguments_start}..args_{arguments_end})")
        }
    }
}
