use crate::{Instruction, Operation};

pub struct Close {
    pub from: u8,
    pub to: u8,
}

impl From<&Instruction> for Close {
    fn from(instruction: &Instruction) -> Self {
        Close {
            from: instruction.b_field(),
            to: instruction.c_field(),
        }
    }
}

impl From<Close> for Instruction {
    fn from(close: Close) -> Self {
        let operation = Operation::Close;
        let (a, b, c) = (0, close.from, close.to);

        Instruction::new(operation, a, b, c, false, false, false)
    }
}
