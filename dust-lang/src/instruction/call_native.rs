use crate::{Destination, Instruction, NativeFunction, Operation};

pub struct CallNative {
    pub destination: Destination,
    pub function: NativeFunction,
    pub argument_count: u16,
}

impl From<&Instruction> for CallNative {
    fn from(instruction: &Instruction) -> Self {
        let destination = if instruction.a_is_local() {
            Destination::Local(instruction.a())
        } else {
            Destination::Register(instruction.a())
        };

        CallNative {
            destination,
            function: NativeFunction::from(instruction.b()),
            argument_count: instruction.c(),
        }
    }
}

impl From<CallNative> for Instruction {
    fn from(call_native: CallNative) -> Self {
        let (a, a_is_local) = match call_native.destination {
            Destination::Local(local) => (local, true),
            Destination::Register(register) => (register, false),
        };

        *Instruction::new(Operation::CallNative)
            .set_a(a)
            .set_a_is_local(a_is_local)
            .set_b(call_native.function as u16)
            .set_c(call_native.argument_count)
    }
}
