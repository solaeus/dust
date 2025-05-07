use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{InstructionFields, TypeCode};

pub struct Close {
    pub from: u16,
    pub to: u16,
    pub r#type: TypeCode,
}

impl From<&Instruction> for Close {
    fn from(instruction: &Instruction) -> Self {
        Close {
            from: instruction.b_field(),
            to: instruction.c_field(),
            r#type: instruction.b_type(),
        }
    }
}

impl From<Close> for Instruction {
    fn from(close: Close) -> Self {
        let operation = Operation::CLOSE;
        let b_field = close.from;
        let b_type = close.r#type;
        let c_field = close.to;

        InstructionFields {
            operation,
            b_field,
            b_type,
            c_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Close {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Close { from, to, r#type } = self;

        match *r#type {
            TypeCode::BOOLEAN => write!(f, "R_BOOL_{from}..=R_BOOL_{to}"),
            TypeCode::BYTE => write!(f, "R_BYTE_{from}..=R_BYTE_{to}"),
            TypeCode::CHARACTER => write!(f, "R_CHAR_{from}..=R_CHAR_{to}"),
            TypeCode::FLOAT => write!(f, "R_FLOAT_{from}..=R_FLOAT_{to}"),
            TypeCode::INTEGER => write!(f, "R_INT_{from}..=R_INT_{to}"),
            TypeCode::STRING => write!(f, "R_STR_{from}..=R_STR_{to}"),
            TypeCode::LIST => write!(f, "R_LIST_{from}..=R_LIST_{to}"),
            unsupported => panic!("Unsupported type code: {unsupported:?}"),
        }
    }
}
