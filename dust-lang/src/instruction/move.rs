use crate::{Instruction, Operation};

pub struct Move {
    pub from: u16,
    pub to: u16,
}

impl From<&Instruction> for Move {
    fn from(instruction: &Instruction) -> Self {
        Move {
            from: instruction.b(),
            to: instruction.a(),
        }
    }
}

impl From<Move> for Instruction {
    fn from(r#move: Move) -> Self {
        *Instruction::new(Operation::Move)
            .set_b(r#move.from)
            .set_c(r#move.to)
    }
}
