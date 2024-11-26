use crate::{Argument, Instruction, Operation};

pub struct Multiply {
    pub destination: u16,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Multiply {
    fn from(instruction: &Instruction) -> Self {
        let (left, right) = instruction.b_and_c_as_arguments();

        Multiply {
            destination: instruction.a(),
            left,
            right,
        }
    }
}

impl From<Multiply> for Instruction {
    fn from(multiply: Multiply) -> Self {
        *Instruction::new(Operation::Multiply)
            .set_a(multiply.destination)
            .set_b(multiply.left.index())
            .set_b_is_constant(multiply.left.is_constant())
            .set_b_is_local(multiply.left.is_local())
            .set_c(multiply.right.index())
            .set_c_is_constant(multiply.right.is_constant())
            .set_c_is_local(multiply.right.is_local())
    }
}
