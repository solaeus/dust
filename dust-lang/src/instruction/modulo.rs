use crate::{Argument, Instruction, Operation};

pub struct Modulo {
    pub destination: u16,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Modulo {
    fn from(instruction: &Instruction) -> Self {
        let (left, right) = instruction.b_and_c_as_arguments();

        Modulo {
            destination: instruction.a(),
            left,
            right,
        }
    }
}

impl From<Modulo> for Instruction {
    fn from(modulo: Modulo) -> Self {
        *Instruction::new(Operation::Modulo)
            .set_a(modulo.destination)
            .set_b(modulo.left.index())
            .set_b_is_constant(modulo.left.is_constant())
            .set_b_is_local(modulo.left.is_local())
            .set_c(modulo.right.index())
            .set_c_is_constant(modulo.right.is_constant())
            .set_c_is_local(modulo.right.is_local())
    }
}
