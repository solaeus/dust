use crate::{Instruction, Operation};

pub struct Move {
    pub from: u8,
    pub to: u8,
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
        let metadata = Operation::Move as u8;
        let a = 0;
        let b = r#move.from;
        let c = r#move.to;

        Instruction { metadata, a, b, c }
    }
}
