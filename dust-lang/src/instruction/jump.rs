use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::InstructionFields;

pub struct Jump {
    pub offset: usize,
    pub is_positive: usize,
}

impl From<Instruction> for Jump {
    fn from(instruction: Instruction) -> Self {
        Jump {
            offset: instruction.b_field(),
            is_positive: instruction.c_field(),
        }
    }
}

impl From<Jump> for Instruction {
    fn from(jump: Jump) -> Self {
        let operation = Operation::JUMP;
        let b_field = jump.offset;
        let c_field = jump.is_positive;

        InstructionFields {
            operation,
            b_field,
            c_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Jump {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Jump {
            offset,
            is_positive,
        } = self;
        let sign = if *is_positive != 0 { "+" } else { "-" };

        write!(f, "JUMP {sign}{offset}")
    }
}
