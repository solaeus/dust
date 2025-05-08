use std::fmt::{self, Display, Formatter};

use super::{Destination, Instruction, InstructionFields, Operand, Operation, TypeCode};

pub struct Modulo {
    pub destination: Destination,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for Modulo {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let left = instruction.b_operand();
        let right = instruction.c_operand();

        Modulo {
            destination,
            left,
            right,
        }
    }
}

impl From<Modulo> for Instruction {
    fn from(modulo: Modulo) -> Self {
        let operation = Operation::MODULO;
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = modulo.destination;
        let Operand {
            index: b_field,
            kind: b_kind,
        } = modulo.left;
        let Operand {
            index: c_field,
            kind: c_kind,
        } = modulo.right;

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

impl Display for Modulo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Modulo {
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
            unsupported => unsupported.unsupported_write(f)?,
        }

        write!(f, " = {left} % {right}",)
    }
}
