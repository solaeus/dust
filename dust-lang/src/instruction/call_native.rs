use std::fmt::Display;

use crate::{Instruction, NativeFunction, Operation, r#type::TypeKind};

use super::{Destination, InstructionFields};

pub struct CallNative {
    pub destination: Destination,
    pub function: NativeFunction,
    pub argument_list_index: u16,
}

impl From<&Instruction> for CallNative {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.destination();
        let function = NativeFunction::from(instruction.b_field());
        let argument_list_index = instruction.c_field();

        CallNative {
            destination,
            function,
            argument_list_index,
        }
    }
}

impl From<CallNative> for Instruction {
    fn from(call_native: CallNative) -> Self {
        let operation = Operation::CALL_NATIVE;
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = call_native.destination;
        let b_field = call_native.function as u16;
        let c_field = call_native.argument_list_index;

        InstructionFields {
            operation,
            a_field,
            a_is_register,
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
            argument_list_index,
        } = self;
        let return_type = function.r#type().return_type.kind();

        if return_type != TypeKind::None {
            destination.display(f, return_type)?;
            write!(f, " = ")?;
        }

        write!(f, "{function}(ARGS_{argument_list_index})")
    }
}
