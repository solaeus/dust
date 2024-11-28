use crate::{Instruction, Operation};

pub struct Close {
    pub from: u16,
    pub to: u16,
}

impl From<&Instruction> for Close {
    fn from(instruction: &Instruction) -> Self {
        Close {
            from: instruction.b(),
            to: instruction.c(),
        }
    }
}

impl From<Close> for Instruction {
    fn from(r#move: Close) -> Self {
        *Instruction::new(Operation::Close)
            .set_b(r#move.from)
            .set_c(r#move.to)
    }
}
