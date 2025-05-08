use std::fmt::{self, Display, Formatter};

use super::{Destination, Instruction, InstructionFields, Operand, Operation, TypeCode};

pub struct Multiply {
    pub destination: Destination,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for Multiply {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let left = instruction.b_operand();
        let right = instruction.c_operand();

        Multiply {
            destination,
            left,
            right,
        }
    }
}

impl From<Multiply> for Instruction {
    fn from(multiply: Multiply) -> Self {
        let operation = Operation::MULTIPLY;
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = multiply.destination;
        let Operand {
            index: b_field,
            kind: b_kind,
        } = multiply.left;
        let Operand {
            index: c_field,
            kind: c_kind,
        } = multiply.right;

        InstructionFields {
            operation,
            a_field,
            a_is_register,
            b_field,
            b_kind,
            c_field,
            c_kind,
        }
        .build()
    }
}

impl Display for Multiply {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Multiply {
            destination,
            left,
            right,
        } = self;

        match left.as_type_code() {
            TypeCode::BOOLEAN => write!(f, "R_BOOL_{}", destination.index)?,
            TypeCode::BYTE => write!(f, "R_BYTE_{}", destination.index)?,
            TypeCode::CHARACTER => write!(f, "R_STR_{}", destination.index)?,
            TypeCode::FLOAT => write!(f, "R_FLOAT_{}", destination.index)?,
            TypeCode::INTEGER => write!(f, "R_INT_{}", destination.index)?,
            TypeCode::STRING => write!(f, "R_STR_{}", destination.index)?,
            _ => todo!(),
        }

        write!(f, " = {left} ✕ {right}",)
    }
}
