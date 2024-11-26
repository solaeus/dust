use crate::{Argument, Instruction};

pub struct Subtract {
    pub destination: u16,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Subtract {
    fn from(instruction: &Instruction) -> Self {
        let (left, right) = instruction.b_and_c_as_arguments();

        Subtract {
            destination: instruction.a(),
            left,
            right,
        }
    }
}

impl From<Subtract> for Instruction {
    fn from(subtract: Subtract) -> Self {
        *Instruction::new(crate::Operation::Subtract)
            .set_a(subtract.destination)
            .set_b(subtract.left.index())
            .set_b_is_constant(subtract.left.is_constant())
            .set_b_is_local(subtract.left.is_local())
            .set_c(subtract.right.index())
            .set_c_is_constant(subtract.right.is_constant())
            .set_c_is_local(subtract.right.is_local())
    }
}
