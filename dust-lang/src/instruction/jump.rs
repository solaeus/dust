use crate::{Instruction, Operation};

pub struct Jump {
    pub offset: u16,
    pub is_positive: bool,
}

impl From<&Instruction> for Jump {
    fn from(instruction: &Instruction) -> Self {
        Jump {
            offset: instruction.b(),
            is_positive: instruction.c_as_boolean(),
        }
    }
}

impl From<Jump> for Instruction {
    fn from(jump: Jump) -> Self {
        *Instruction::new(Operation::Jump)
            .set_b(jump.offset)
            .set_c_to_boolean(jump.is_positive)
    }
}
