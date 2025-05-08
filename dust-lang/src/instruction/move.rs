use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{Destination, InstructionFields, Operand, TypeCode};

pub struct Move {
    pub destination: Destination,
    pub operand: Operand,
}

impl From<&Instruction> for Move {
    fn from(instruction: &Instruction) -> Self {
        Move {
            destination: instruction.destination(),
            operand: instruction.b_operand(),
        }
    }
}

impl From<Move> for Instruction {
    fn from(r#move: Move) -> Self {
        let operation = Operation::MOVE;
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = r#move.destination;
        let Operand {
            index: b_field,
            kind: b_kind,
        } = r#move.operand;

        InstructionFields {
            operation,
            a_field,
            a_is_register,
            b_field,
            b_kind,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Move {
            destination,
            operand,
        } = self;

        match operand.as_type_code() {
            TypeCode::BOOLEAN => write!(f, "R_BOOL_{}", destination.index)?,
            TypeCode::BYTE => write!(f, "R_BYTE_{}", destination.index)?,
            TypeCode::CHARACTER => write!(f, "R_CHAR_{}", destination.index)?,
            TypeCode::FLOAT => write!(f, "R_FLOAT_{}", destination.index)?,
            TypeCode::INTEGER => write!(f, "R_INT_{}", destination.index)?,
            TypeCode::STRING => write!(f, "R_STR_{}", destination.index)?,
            TypeCode::LIST => write!(f, "R_LIST_{}", destination.index)?,
            TypeCode::FUNCTION => write!(f, "R_FN_{}", destination.index)?,
            unsupported => unsupported.unsupported_write(f)?,
        }

        write!(f, " = {operand}")
    }
}
