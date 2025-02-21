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
        let b_type = r#move.operand.as_type();

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
            operand: to,
        } = self;

        match to.as_type() {
            TypeCode::BOOLEAN => write!(f, "R_BOOL_{destination} -> {to}"),
            TypeCode::BYTE => write!(f, "R_BYTE_{destination} -> {to}"),
            TypeCode::CHARACTER => write!(f, "R_CHAR_{destination} -> {to}"),
            TypeCode::FLOAT => write!(f, "R_FLOAT_{destination} -> {to}"),
            TypeCode::INTEGER => write!(f, "R_INT_{destination} -> {to}"),
            TypeCode::STRING => write!(f, "R_STR_{destination} -> {to}"),
            TypeCode::LIST => write!(f, "R_LIST_{destination} -> {to}"),
            TypeCode::FUNCTION => write!(f, "R_FN_{destination} -> {to}"),
            unsupported => write!(
                f,
                "Unsupported type code: {unsupported} for MOVE instruction"
            ),
        }
    }
}
