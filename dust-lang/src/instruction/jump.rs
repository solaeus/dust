use crate::{Instruction, Operation};

pub struct Jump {
    pub offset: u8,
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
        let metadata = Operation::Jump as u8;
        let a = 0;
        let b = jump.offset;
        let c = jump.is_positive as u8;

        Instruction { metadata, a, b, c }
    }
}
