use crate::{Instruction, Operation};

use super::InstructionBuilder;

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

        InstructionBuilder {
            operation,
            b_field,
            c_field,
            ..Default::default()
        }
        .build()
    }
}
