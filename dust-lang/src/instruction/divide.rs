use crate::{Argument, Instruction, Operation};

pub struct Divide {
    pub destination: u16,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Divide {
    fn from(instruction: &Instruction) -> Self {
        let (left, right) = instruction.b_and_c_as_arguments();

        Divide {
            destination: instruction.a(),
            left,
            right,
        }
    }
}

impl From<Divide> for Instruction {
    fn from(divide: Divide) -> Self {
        *Instruction::new(Operation::Divide)
            .set_a(divide.destination)
            .set_b(divide.left.index())
            .set_b_is_constant(divide.left.is_constant())
            .set_b_is_local(divide.left.is_local())
            .set_c(divide.right.index())
            .set_c_is_constant(divide.right.is_constant())
            .set_c_is_local(divide.right.is_local())
    }
}
