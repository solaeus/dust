use std::fmt::{self, Display, Formatter};

use super::{Destination, Instruction, InstructionFields, Operand, Operation, TypeCode};

pub struct Add {
    pub destination: Destination,
    pub left: Operand,
    pub right: Operand,
}

impl From<&Instruction> for Add {
    fn from(instruction: &Instruction) -> Self {
        let destination = Destination {
            index: instruction.a_field(),
            is_register: instruction.a_is_register(),
        };
        let left = instruction.b_operand();
        let right = instruction.c_operand();

        Add {
            destination,
            left,
            right,
        }
    }
}

impl From<Add> for Instruction {
    fn from(add: Add) -> Self {
        let operation = Operation::ADD;
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = add.destination;
        let Operand {
            index: b_field,
            kind: b_kind,
        } = add.left;
        let Operand {
            index: c_field,
            kind: c_kind,
        } = add.right;

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

impl Display for Add {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Add {
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

        write!(f, " = {left} + {right}")
    }
}
