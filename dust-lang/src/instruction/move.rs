use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation, r#type::TypeKind};

use super::{Address, Destination, InstructionFields};

pub struct Move {
    pub destination: Destination,
    pub operand: Address,
}

impl From<&Instruction> for Move {
    fn from(instruction: &Instruction) -> Self {
        Move {
            destination: instruction.destination(),
            operand: instruction.b_address(),
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
        let Address {
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

        match operand.r#type() {
            TypeKind::Boolean => write!(f, "R_BOOL_{}", destination.index)?,
            TypeKind::Byte => write!(f, "R_BYTE_{}", destination.index)?,
            TypeKind::Character => write!(f, "R_CHAR_{}", destination.index)?,
            TypeKind::Float => write!(f, "R_FLOAT_{}", destination.index)?,
            TypeKind::Integer => write!(f, "R_INT_{}", destination.index)?,
            TypeKind::String => write!(f, "R_STR_{}", destination.index)?,
            TypeKind::List => write!(f, "R_LIST_{}", destination.index)?,
            TypeKind::Function => write!(f, "R_FN_{}", destination.index)?,
            invalid => invalid.write_invalid(f)?,
        }

        write!(f, " = {operand}")
    }
}
