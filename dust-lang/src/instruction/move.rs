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
            from: instruction.b_field(),
            to: instruction.c_field(),
            type_code: instruction.b_type(),
        }
    }
}

impl From<Move> for Instruction {
    fn from(r#move: Move) -> Self {
        let operation = Operation::MOVE;
        let b_field = r#move.from;
        let c_field = r#move.to;
        let b_type = r#move.type_code;

        InstructionBuilder {
            operation,
            b_field,
            b_type,
            c_field,
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
