use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{TwoOperandLayout, TypeCode};

pub struct LoadConstant {
    pub destination: u16,
    pub type_code: TypeCode,
    pub constant_index: u16,
    pub jump_next: bool,
}

impl From<Instruction> for LoadConstant {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let type_code = instruction.b_type();
        let constant_index = instruction.b_field();
        let jump_next = instruction.c_field() != 0;

        LoadConstant {
            destination,
            type_code,
            constant_index,
            jump_next,
        }
    }
}

impl From<LoadConstant> for Instruction {
    fn from(load_constant: LoadConstant) -> Self {
        TwoOperandLayout {
            operation: Operation::LOAD_CONSTANT,
            a_field: load_constant.destination,
            b_type: load_constant.type_code,
            b_field: load_constant.constant_index,
            c_field: load_constant.jump_next as u16,
            ..Default::default()
        }
        .build()
    }
}

impl Display for LoadConstant {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LoadConstant {
            destination,
            type_code,
            constant_index,
            jump_next,
        } = self;

        match *type_code {
            TypeCode::BOOLEAN => write!(f, "R_BOOL_{destination} = C_BOOL_{constant_index}")?,
            TypeCode::BYTE => write!(f, "R_BYTE_{destination} = C_BYTE_{constant_index}")?,
            TypeCode::CHARACTER => write!(f, "R_CHAR_{destination} = C_CHAR_{constant_index}")?,
            TypeCode::FLOAT => write!(f, "R_FLOAT_{destination} = C_FLOAT_{constant_index}")?,
            TypeCode::INTEGER => write!(f, "R_INT_{destination} = C_INT_{constant_index}")?,
            TypeCode::STRING => write!(f, "R_STR_{destination} = C_STR_{constant_index}")?,
            unknown => unknown.panic_from_unknown_code(),
        }

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
