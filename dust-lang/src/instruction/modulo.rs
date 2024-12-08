use crate::{Argument, Destination, Instruction, Operation};

pub struct Modulo {
    pub destination: Destination,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Modulo {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_as_destination();
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
        let (a, a_options) = modulo.destination.as_index_and_a_options();
        let (b, b_options) = modulo.left.as_index_and_b_options();
        let (c, c_options) = modulo.right.as_index_and_c_options();

        Instruction {
            operation: Operation::MODULO,
            options: a_options | b_options | c_options,
            a,
            b,
            c,
        }
    }
}
