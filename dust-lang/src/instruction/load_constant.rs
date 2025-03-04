use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{InstructionFields, TypeCode};

pub struct LoadConstant {
    pub destination: u16,
    pub constant_index: u16,
    pub constant_type: TypeCode,
    pub jump_next: bool,
}

impl From<Instruction> for LoadConstant {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let constant_index = instruction.b_field();
        let constant_type = instruction.b_type();
        let jump_next = instruction.c_field() != 0;

        LoadConstant {
            destination,
            constant_index,
            constant_type,
            jump_next,
        }
    }
}

impl From<LoadConstant> for Instruction {
    fn from(load_constant: LoadConstant) -> Self {
        InstructionFields {
            operation: Operation::LOAD_CONSTANT,
            a_field: load_constant.destination,
            b_field: load_constant.constant_index,
            b_type: load_constant.constant_type,
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
            constant_index,
            constant_type,
            jump_next,
        } = self;

        match *constant_type {
            TypeCode::CHARACTER => write!(f, "R_CHAR_{destination} = C_CHAR_{constant_index}")?,
            TypeCode::FLOAT => write!(f, "R_FLOAT_{destination} = C_FLOAT_{constant_index}")?,
            TypeCode::INTEGER => write!(f, "R_INT_{destination} = C_INT_{constant_index}")?,
            TypeCode::STRING => write!(f, "R_STR_{destination} = C_STR_{constant_index}")?,
            unsupported => panic!("Unsupported type code: {}", unsupported.0),
        }

        if *jump_next {
            write!(f, " JUMP +1")?;
        }

        Ok(())
    }
}
