use crate::{Instruction, Operation};

pub struct Close {
    pub from: u8,
    pub to: u8,
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
        let metadata = Operation::Close as u8;
        let (a, b, c) = (0, close.from, close.to);

        Instruction { metadata, a, b, c }
    }
}
