use std::fmt::Display;

use crate::{Instruction, NativeFunction, Operation};

use super::InstructionBuilder;

pub struct CallNative {
    pub destination: u16,
    pub function: NativeFunction,
    pub argument_count: u16,
}

impl From<Instruction> for CallNative {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let function = NativeFunction::from(instruction.b_field());

        CallNative {
            destination,
            function,
            argument_count: instruction.c_field(),
        }
    }
}

impl From<CallNative> for Instruction {
    fn from(call_native: CallNative) -> Self {
        let operation = Operation::CALL_NATIVE;
        let a_field = call_native.destination;
        let b_field = call_native.function as u16;
        let c_field = call_native.argument_count;

        InstructionBuilder {
            operation,
            a_field,
            b_field,
            c_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for CallNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let CallNative {
            destination,
            function,
            argument_count,
        } = self;
        let arguments_start = destination.saturating_sub(*argument_count);
        let arguments_end = arguments_start + argument_count;

        if function.returns_value() {
            write!(f, "R{destination} = ")?;
        }

        match argument_count {
            0 => {
                write!(f, "{function}()")
            }
            1 => write!(f, "{function}(R{arguments_start})"),
            _ => write!(f, "{function}(R{arguments_start}..R{arguments_end})"),
        }
    }
}
