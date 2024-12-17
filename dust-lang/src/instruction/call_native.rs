use crate::{Instruction, NativeFunction, Operation};

use super::InstructionData;

pub struct CallNative {
    pub destination: u8,
    pub function: NativeFunction,
    pub argument_count: u8,
}

impl From<&Instruction> for CallNative {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let function = NativeFunction::from(instruction.b_field());

        CallNative {
            destination,
            function,
            argument_count: instruction.c_field(),
        }
    }
}

impl From<InstructionData> for CallNative {
    fn from(data: InstructionData) -> Self {
        CallNative {
            destination: data.a_field,
            function: NativeFunction::from(data.b_field),
            argument_count: data.c_field,
        }
    }
}

impl From<CallNative> for Instruction {
    fn from(call_native: CallNative) -> Self {
        let operation = Operation::CALL_NATIVE;
        let a = call_native.destination;
        let b = call_native.function as u8;
        let c = call_native.argument_count;

        Instruction::new(operation, a, b, c, false, false, false)
    }
}
