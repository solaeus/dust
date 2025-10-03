use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionFields, Operation};

pub struct Jump {
    pub offset: u16,
    pub is_positive: bool,
}

impl From<Instruction> for Jump {
    fn from(instruction: Instruction) -> Self {
        Jump {
            offset: instruction.b_field(),
            is_positive: instruction.c_field() != 0,
        }
    }
}

impl From<Jump> for Instruction {
    fn from(jump: Jump) -> Self {
        let operation = Operation::JUMP;
        let b_field = jump.offset;
        let c_field = jump.is_positive as u16;

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
        let sign = if *is_positive { "+" } else { "-" };

        write!(f, "jump {sign}{offset}")
    }
}
