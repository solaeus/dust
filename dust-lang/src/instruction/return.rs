use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{InstructionBuilder, TypeCode};

pub struct Return {
    pub should_return_value: bool,
    pub return_type: TypeCode,
    pub return_register: u16,
}

impl From<Instruction> for Return {
    fn from(instruction: Instruction) -> Self {
        let should_return_value = instruction.b_field() != 0;
        let return_type = instruction.b_type();
        let return_register = instruction.c_field();

        Return {
            should_return_value,
            return_type,
            return_register,
        }
    }
}

impl From<Return> for Instruction {
    fn from(r#return: Return) -> Self {
        let operation = Operation::RETURN;
        let b_field = r#return.should_return_value as u16;
        let b_type = r#return.return_type;
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
            return_type,
            return_register,
        } = self;

        if *should_return_value {
            match *return_type {
                TypeCode::BOOLEAN => write!(f, "RETURN R_BOOL_{}", return_register),
                TypeCode::BYTE => write!(f, "RETURN R_BYTE_{}", return_register),
                TypeCode::CHARACTER => write!(f, "RETURN R_CHAR_{}", return_register),
                TypeCode::INTEGER => write!(f, "RETURN R_INT_{}", return_register),
                TypeCode::FLOAT => write!(f, "RETURN R_FLOAT_{}", return_register),
                TypeCode::STRING => write!(f, "RETURN R_STRING_{}", return_register),
                unknown => unknown.panic_from_unknown_code(),
            }
        } else {
            write!(f, "RETURN")
        }
    }
}
