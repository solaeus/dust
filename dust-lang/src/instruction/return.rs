use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{InstructionBuilder, TypeCode};

pub struct Return {
    pub should_return_value: bool,
    pub return_register: u16,
    pub r#type: TypeCode,
}

impl From<Instruction> for Return {
    fn from(instruction: Instruction) -> Self {
        let should_return_value = instruction.b_field() != 0;
        let return_register = instruction.c_field();
        let r#type = instruction.b_type();

        Return {
            should_return_value,
            return_register,
            r#type,
        }
    }
}

impl From<Return> for Instruction {
    fn from(r#return: Return) -> Self {
        let operation = Operation::RETURN;
        let b_field = r#return.should_return_value as u16;
        let b_type = r#return.r#type;
        let c_field = r#return.return_register;

        InstructionBuilder {
            operation,
            b_field,
            b_type,
            c_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Return {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Return {
            should_return_value,
            return_register,
            r#type,
        } = self;

        if *should_return_value {
            match *r#type {
                TypeCode::BOOLEAN => write!(f, "RETURN R_BOOL_{return_register}"),
                TypeCode::BYTE => write!(f, "RETURN R_BYTE_{return_register}"),
                TypeCode::CHARACTER => write!(f, "RETURN R_CHAR_{return_register}"),
                TypeCode::FLOAT => write!(f, "RETURN R_FLOAT_{return_register}"),
                TypeCode::INTEGER => write!(f, "RETURN R_INT_{return_register}"),
                TypeCode::STRING => write!(f, "RETURN R_STR_{return_register}"),
                unsupported => unreachable!("Unsupported return type: {:?}", unsupported),
            }
        } else {
            write!(f, "RETURN")
        }
    }
}
