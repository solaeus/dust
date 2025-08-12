use std::fmt::Display;

use crate::NativeFunction;

use super::{Address, Instruction, InstructionFields, Operation};

pub struct CallNative {
    pub destination: Address,
    pub function: NativeFunction,
    pub arguments_index: u16,
}

impl From<Instruction> for CallNative {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let function = NativeFunction::from_index(instruction.b_field());
        let arguments_index = instruction.c_field();

        CallNative {
            destination,
            function,
            arguments_index,
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
        let c_field = call_native.arguments_index;

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
            arguments_index: argument_index,
        } = self;

        if function.returns_value() {
            let return_type = function.r#type().return_type.as_operand_type();

            destination.display(f, return_type)?;
            write!(f, " = ")?;
        }

        write!(f, "{function}(args_{argument_index})")
    }
}
