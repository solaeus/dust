use std::fmt::Display;

use crate::{Chunk, NativeFunction};

use super::{Address, Instruction, InstructionFields, Operation};

pub struct CallNative<C> {
    pub destination: Address,
    pub function: NativeFunction<C>,
    pub argument_count: usize,
}

impl<C: Chunk> From<&Instruction> for CallNative<C> {
    fn from(instruction: &Instruction) -> Self {
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

impl<C> From<CallNative<C>> for Instruction {
    fn from(call_native: CallNative<C>) -> Self {
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

impl<C: Chunk> Display for CallNative<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let CallNative {
            destination,
            function,
            argument_count,
        } = self;

        write!(f, "{destination} = {function}")?;

        if *argument_count > 0 {
            write!(
                f,
                "({}..={})",
                function.index + 1,
                function.index + argument_count + 1
            )
        } else {
            write!(f, "()")
        }
    }
}
