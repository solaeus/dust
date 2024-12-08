use crate::{Instruction, Operation};

use super::InstructionOptions;

pub struct Jump {
    pub offset: u16,
    pub is_positive: bool,
}

impl From<&Instruction> for Jump {
    fn from(instruction: &Instruction) -> Self {
        Jump {
            offset: instruction.b,
            is_positive: instruction.c != 0,
        }
    }
}

impl From<Jump> for Instruction {
    fn from(jump: Jump) -> Self {
        Instruction {
            operation: Operation::JUMP,
            options: InstructionOptions::empty(),
            a: 0,
            b: jump.offset,
            c: jump.is_positive as u16,
        }
    }
}
