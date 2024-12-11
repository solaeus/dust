use crate::{Instruction, Operation};

pub struct Jump {
    pub offset: u8,
    pub is_positive: bool,
}

impl From<&Instruction> for Jump {
    fn from(instruction: &Instruction) -> Self {
        Jump {
            offset: instruction.b_field(),
            is_positive: instruction.c_field() != 0,
        }
    }
}

impl From<Jump> for Instruction {
    fn from(jump: Jump) -> Self {
        let operation = Operation::Jump;
        let b = jump.offset;
        let c = jump.is_positive as u8;

        Instruction::new(operation, 0, b, c, false, false, false)
    }
}
