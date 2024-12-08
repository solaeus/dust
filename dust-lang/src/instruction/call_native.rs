use crate::{Destination, Instruction, NativeFunction, Operation};

pub struct CallNative {
    pub destination: Destination,
    pub function: NativeFunction,
    pub argument_count: u16,
}

impl From<&Instruction> for CallNative {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_as_destination();
        let function = NativeFunction::from(instruction.b);

        CallNative {
            destination,
            function,
            argument_count: instruction.c,
        }
    }
}

impl From<CallNative> for Instruction {
    fn from(call_native: CallNative) -> Self {
        let (a, a_options) = call_native.destination.as_index_and_a_options();
        let b = call_native.function as u16;

        Instruction {
            operation: Operation::CALL_NATIVE,
            options: a_options,
            a,
            b,
            c: call_native.argument_count,
        }
    }
}
