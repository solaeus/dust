use crate::{Argument, Instruction, Operation};

pub struct Modulo {
    pub destination: u8,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Modulo {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_arguments();

        Modulo {
            destination,
            left,
            right,
        }
    }
}

impl From<Modulo> for Instruction {
    fn from(modulo: Modulo) -> Self {
        let operation = Operation::Modulo;
        let a = modulo.destination;
        let (b, b_is_constant) = modulo.left.as_index_and_constant_flag();
        let (c, c_is_constant) = modulo.right.as_index_and_constant_flag();

        Instruction::new(operation, a, b, c, b_is_constant, c_is_constant, false)
    }
}
