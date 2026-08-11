use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, NativeFunction, Operation};

pub struct CallNative {
    pub destination: u16,
    pub function: NativeFunction,
    pub arguments_start: u16,
}

impl From<Instruction> for CallNative {
    fn from(instruction: Instruction) -> Self {
        CallNative {
            destination: instruction.a_field() as u16,
            function: NativeFunction::from_u16(instruction.b_field() as u16),
            arguments_start: instruction.c_field() as u16,
        }
    }
}

impl From<CallNative> for Instruction {
    fn from(call_native: CallNative) -> Self {
        let CallNative {
            destination,
            function,
            arguments_start,
        } = call_native;

        InstructionBuilder::new(Operation::CALL_NATIVE)
            .a_field(destination)
            .b_field(function.as_u16())
            .c_field(arguments_start)
            .build()
    }
}

impl Display for CallNative {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let CallNative {
            destination,
            function,
            arguments_start,
        } = *self;

        if destination != 0 {
            write!(f, "reg_{destination} = {function}")?;
        }

        if arguments_start == u16::MAX {
            write!(f, "()")
        } else {
            write!(f, "(reg_{arguments_start}...)")
        }
    }
}
