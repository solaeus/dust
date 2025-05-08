use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{InstructionFields, Operand, TypeCode};

pub struct Move {
    pub destination: u16,
    pub operand: Operand,
}

impl From<&Instruction> for Move {
    fn from(instruction: &Instruction) -> Self {
        Move {
            destination: instruction.a_field(),
            operand: instruction.b_as_operand(),
        }
    }
}

impl From<Move> for Instruction {
    fn from(r#move: Move) -> Self {
        let operation = Operation::MOVE;
        let a_field = r#move.destination;
        let (b_field, b_is_constant) = r#move.operand.as_index_and_constant_flag();
        let b_type = r#move.operand.as_type_code();

        InstructionFields {
            operation,
            a_field,
            b_field,
            b_type,
            b_is_constant,
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
            TypeCode::BOOLEAN => write!(f, "{operand} -> R_BOOL_{destination}"),
            TypeCode::BYTE => write!(f, "{operand} -> R_BYTE_{destination}"),
            TypeCode::CHARACTER => write!(f, "{operand} -> R_CHAR_{destination}"),
            TypeCode::FLOAT => write!(f, "{operand} -> R_FLOAT_{destination}"),
            TypeCode::INTEGER => write!(f, "{operand} -> R_INT_{destination}"),
            TypeCode::STRING => write!(f, "{operand} -> R_STR_{destination}"),
            TypeCode::LIST => write!(f, "{operand} -> R_LIST_{destination}"),
            TypeCode::FUNCTION => write!(f, "{operand} -> R_FN_{destination}"),
            unsupported => write!(
                f,
                "Unsupported type code: {unsupported} for MOVE instruction"
            ),
        }
    }
}
