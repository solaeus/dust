use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{InstructionFields, Operand, TypeCode};

pub struct Return {
    pub should_return_value: bool,
    pub return_value: Operand,
}

impl From<Instruction> for Return {
    fn from(instruction: Instruction) -> Self {
        let should_return_value = instruction.a_field() != 0;
        let return_value = instruction.b_operand();

        Return {
            should_return_value,
            return_value,
        }
    }
}

impl From<Return> for Instruction {
    fn from(r#return: Return) -> Self {
        let operation = Operation::RETURN;
        let a_field = r#return.should_return_value as u16;
        let Operand {
            index: b_field,
            kind: b_kind,
        } = r#return.return_value;

        InstructionFields {
            operation,
            a_field,
            b_field,
            b_kind,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Return {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Return {
            should_return_value,
            return_value,
        } = self;
        write!(f, "RETURN")?;

        if *should_return_value {
            match return_value.as_type_code() {
                TypeCode::BOOLEAN => write!(f, " R_BOOL_{}", return_value.index)?,
                TypeCode::BYTE => write!(f, " R_BYTE_{}", return_value.index)?,
                TypeCode::CHARACTER => write!(f, " R_CHAR_{}", return_value.index)?,
                TypeCode::FLOAT => write!(f, " R_FLOAT_{}", return_value.index)?,
                TypeCode::INTEGER => write!(f, " R_INT_{}", return_value.index)?,
                TypeCode::STRING => write!(f, " R_STR_{}", return_value.index)?,
                TypeCode::LIST => write!(f, " R_LIST_{}", return_value.index)?,
                TypeCode::FUNCTION => write!(f, " R_FN_{}", return_value.index)?,
                unsupported => unsupported.unsupported_write(f)?,
            }
        }

        Ok(())
    }
}
