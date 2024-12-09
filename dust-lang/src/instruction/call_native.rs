use crate::{Instruction, NativeFunction, Operation};

pub struct CallNative {
    pub destination: u8,
    pub function: NativeFunction,
    pub argument_count: u8,
}

impl From<&Instruction> for CallNative {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a;
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
        let metadata = Operation::CallNative as u8;
        let a = call_native.destination;
        let b = call_native.function as u8;
        let c = call_native.argument_count;

        Instruction { metadata, a, b, c }
    }
}
