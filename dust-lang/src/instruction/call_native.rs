use crate::{Instruction, NativeFunction, Operation};

pub struct CallNative {
    pub destination: u16,
    pub function: NativeFunction,
    pub argument_count: u16,
}

impl From<&Instruction> for CallNative {
    fn from(instruction: &Instruction) -> Self {
        CallNative {
            destination: instruction.a(),
            function: NativeFunction::from(instruction.b()),
            argument_count: instruction.c(),
        }
    }
}

impl From<CallNative> for Instruction {
    fn from(call_native: CallNative) -> Self {
        *Instruction::new(Operation::CallNative)
            .set_a(call_native.destination)
            .set_b(call_native.function as u16)
            .set_c(call_native.argument_count)
    }
}
