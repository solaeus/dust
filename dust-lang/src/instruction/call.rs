use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{InstructionFields, TypeCode};

pub struct Call {
    pub destination: u16,
    pub function_register: u16,
    pub argument_list_index: u16,
    pub return_type: TypeCode,
}

impl From<Instruction> for Call {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let function_register = instruction.b_field();
        let argument_list_index = instruction.c_field();
        let return_type = instruction.b_type();

        Call {
            destination,
            function_register,
            argument_list_index,
            return_type,
        }
    }
}

impl From<Call> for Instruction {
    fn from(call: Call) -> Self {
        let a_field = call.destination;
        let b_field = call.function_register;
        let b_type = call.return_type;
        let c_field = call.argument_list_index;

        InstructionFields {
            operation: Operation::CALL,
            a_field,
            b_field,
            b_type,
            c_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Call {
            destination,
            function_register,
            argument_list_index,
            return_type,
            ..
        } = self;

        match *return_type {
            TypeCode::NONE => {}
            TypeCode::BOOLEAN => write!(f, "R_BOOL_{destination} = ")?,
            TypeCode::BYTE => write!(f, "R_BYTE_{destination} = ")?,
            TypeCode::CHARACTER => write!(f, "R_CHR_{destination} = ")?,
            TypeCode::FLOAT => write!(f, "R_FLT_{destination} = ")?,
            TypeCode::INTEGER => write!(f, "R_INT_{destination} = ")?,
            TypeCode::STRING => write!(f, "R_STR_{destination} = ")?,
            TypeCode::LIST => write!(f, "R_LIST_{destination} = ")?,
            TypeCode::FUNCTION => write!(f, "R_FN_{destination} = ")?,
            _ => unreachable!(),
        }

        write!(f, "R_FN_{function_register}({argument_list_index})")
    }
}
