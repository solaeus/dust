use std::fmt::{self, Display, Formatter};

use crate::{
    instruction::{Instruction, InstructionBuilder, OperandType, Operation},
    native_function::NativeFunction,
};

pub struct CallNative {
    pub destination: u16,
    pub function: NativeFunction,
    pub argument_type: OperandType,
    pub arguments_start: u16,
}

impl From<Instruction> for CallNative {
    fn from(instruction: Instruction) -> Self {
        CallNative {
            destination: instruction.a_field(),
            function: NativeFunction::from_id(instruction.b_field()),
            argument_type: instruction.operand_type(),
            arguments_start: instruction.c_field(),
        }
    }
}

impl From<CallNative> for Instruction {
    fn from(call_native: CallNative) -> Self {
        let CallNative {
            destination,
            function,
            argument_type,
            arguments_start,
        } = call_native;

        InstructionBuilder::new(Operation::CALL_NATIVE)
            .a_field(destination)
            .b_field(function.id())
            .operand_type(argument_type)
            .c_field(arguments_start)
            .build()
    }
}

impl Display for CallNative {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let CallNative {
            destination,
            function,
            argument_type: _,
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
