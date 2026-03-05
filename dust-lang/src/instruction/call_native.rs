use std::fmt::{self, Display, Formatter};

use crate::native_function::NativeFunction;

use super::{Instruction, InstructionBuilder, Operation};

pub struct CallNative {
    pub destination: u16,
    pub function_id: u16,
    pub arguments_start: u16,
    pub argument_count: u16,
}

impl From<&Instruction> for CallNative {
    fn from(instruction: &Instruction) -> Self {
        CallNative {
            destination: instruction.a_field(),
            function_id: instruction.b_field(),
            arguments_start: instruction.c_field(),
            argument_count: instruction.d_field(),
        }
    }
}

impl From<CallNative> for Instruction {
    fn from(call_native: CallNative) -> Self {
        let CallNative {
            destination,
            function_id,
            arguments_start,
            argument_count: arguments_count,
        } = call_native;

        InstructionBuilder::new(Operation::CALL_NATIVE)
            .a_field(destination)
            .b_field(function_id)
            .c_field(arguments_start)
            .d_field(arguments_count)
            .build()
    }
}

impl Display for CallNative {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let CallNative {
            destination,
            function_id,
            arguments_start,
            argument_count,
        } = *self;
        let function = NativeFunction { id: function_id };

        if destination != 0 {
            write!(f, "reg_{destination} = ")?;
        }

        if argument_count == 0 {
            write!(f, "{function}()")
        } else {
            let arguments_end = arguments_start + argument_count;

            write!(
                f,
                "{function}(args_{arguments_start}..args_{arguments_end})"
            )
        }
    }
}
