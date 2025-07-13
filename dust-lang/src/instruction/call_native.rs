use std::fmt::Display;

use crate::NativeFunction;

use super::{Address, Instruction, InstructionFields, Operation};

pub struct CallNative {
    pub destination: Address,
    pub function: NativeFunction,
    pub argument_count: usize,
}

impl From<Instruction> for CallNative {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let function = NativeFunction::from_index(instruction.b_field());
        let argument_count = instruction.c_field();

        CallNative {
            destination,
            function,
            argument_count,
        }
    }
}

impl From<CallNative> for Instruction {
    fn from(call_native: CallNative) -> Self {
        let operation = Operation::CALL_NATIVE;
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = call_native.destination;
        let b_field = call_native.function.index;
        let c_field = call_native.argument_count;

        InstructionFields {
            operation,
            a_field,
            a_memory_kind,
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

        if function.returns_value() {
            write!(f, "{destination} = ")?;
        }

        write!(f, "{function}")?;

        if *argument_count == 1 {
            write!(
                f,
                "({})",
                Address::register(destination.index - argument_count)
            )
        } else if *argument_count > 0 {
            write!(
                f,
                "({}..{})",
                Address::register(destination.index - argument_count),
                Address::register(destination.index)
            )
        } else {
            write!(f, "()")
        }
    }
}
