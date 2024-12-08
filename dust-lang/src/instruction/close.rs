use crate::{Instruction, Operation};

use super::InstructionOptions;

pub struct Close {
    pub from: u16,
    pub to: u16,
}

impl From<&Instruction> for Close {
    fn from(instruction: &Instruction) -> Self {
        Close {
            from: instruction.b,
            to: instruction.c,
        }
    }
}

impl From<Close> for Instruction {
    fn from(close: Close) -> Self {
        Instruction {
            operation: Operation::CLOSE,
            options: InstructionOptions::empty(),
            a: 0,
            b: close.from,
            c: close.to,
        }
    }
}
