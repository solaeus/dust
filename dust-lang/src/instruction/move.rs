use crate::{Instruction, Operation};

use super::InstructionOptions;

pub struct Move {
    pub from: u16,
    pub to: u16,
}

impl From<&Instruction> for Move {
    fn from(instruction: &Instruction) -> Self {
        Move {
            from: instruction.b,
            to: instruction.c,
        }
    }
}

impl From<Move> for Instruction {
    fn from(r#move: Move) -> Self {
        Instruction {
            operation: Operation::MOVE,
            options: InstructionOptions::empty(),
            a: 0,
            b: r#move.from,
            c: r#move.to,
        }
    }
}
