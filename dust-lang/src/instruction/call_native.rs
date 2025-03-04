use std::fmt::Display;

use crate::{Instruction, NativeFunction, Operation, Type};

use super::InstructionFields;

pub struct CallNative {
    pub destination: u16,
    pub function: NativeFunction,
    pub argument_list_index: u16,
}

impl From<Instruction> for CallNative {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let function = NativeFunction::from(instruction.b_field());
        let first_argument_index = instruction.c_field();

        CallNative {
            destination,
            function,
            argument_list_index: first_argument_index,
        }
    }
}

impl From<CallNative> for Instruction {
    fn from(call_native: CallNative) -> Self {
        let operation = Operation::CALL_NATIVE;
        let a_field = call_native.destination;
        let b_field = call_native.function as u16;
        let c_field = call_native.argument_list_index;

        InstructionFields {
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
            argument_list_index,
        } = self;
        let return_type = function.r#type().return_type;

        match *return_type {
            Type::None => {}
            Type::Boolean => write!(f, "R_BOOL_{destination} = ")?,
            Type::Byte => write!(f, "R_BYTE_{destination} = ")?,
            Type::Character => write!(f, "R_CHR_{destination} = ")?,
            Type::Float => write!(f, "R_FLT_{destination} = ")?,
            Type::Integer => write!(f, "R_INT_{destination} = ")?,
            Type::String => write!(f, "R_STR_{destination} = ")?,
            Type::List(_) => write!(f, "R_LIST_{destination} = ")?,
            Type::Function(_) => write!(f, "R_FN_{destination} = ")?,
            _ => unreachable!(),
        }

        write!(f, "{function}(ARGS_{argument_list_index})")
    }
}
