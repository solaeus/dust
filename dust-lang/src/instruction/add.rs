use crate::{Argument, Instruction, Operation};

pub struct Add {
    pub destination: u16,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Add {
    fn from(instruction: &Instruction) -> Self {
        let (left, right) = instruction.b_and_c_as_arguments();

        Add {
            destination: instruction.a(),
            left,
            right,
        }
    }
}

impl From<Add> for Instruction {
    fn from(add: Add) -> Self {
        *Instruction::new(Operation::Add)
            .set_a(add.destination)
            .set_b(add.left.index())
            .set_b_is_constant(add.left.is_constant())
            .set_b_is_local(add.left.is_local())
            .set_c(add.right.index())
            .set_c_is_constant(add.right.is_constant())
            .set_c_is_local(add.right.is_local())
    }
}
