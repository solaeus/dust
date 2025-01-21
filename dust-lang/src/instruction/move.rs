use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{InstructionBuilder, TypeCode};

pub struct Move {
    pub from: u16,
    pub to: u16,
    pub type_code: TypeCode,
}

impl From<Instruction> for Move {
    fn from(instruction: Instruction) -> Self {
        Move {
            from: instruction.a_field(),
            to: instruction.b_field(),
            type_code: instruction.b_type(),
        }
    }
}

impl From<Move> for Instruction {
    fn from(r#move: Move) -> Self {
        let operation = Operation::MOVE;
        let a_field = r#move.from;
        let b_field = r#move.to;
        let b_type = r#move.type_code;

        InstructionBuilder {
            operation,
            a_field,
            b_field,
            b_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Move {
            from,
            to,
            type_code,
        } = self;
        let register_name = type_code.register_name();
        let operator = if let TypeCode::STRING = *type_code {
            "->"
        } else {
            "="
        };

        write!(f, "{register_name}_{from} {operator} {register_name}_{to}")
    }
}
