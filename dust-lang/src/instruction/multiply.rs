use crate::{Argument, Destination, Instruction, Operation};

pub struct Multiply {
    pub destination: Destination,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Multiply {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_as_destination();
        let (left, right) = instruction.b_and_c_as_arguments();

        Multiply {
            destination,
            left,
            right,
        }
    }
}

impl From<Multiply> for Instruction {
    fn from(multiply: Multiply) -> Self {
        let (a, a_options) = multiply.destination.as_index_and_a_options();
        let (b, b_options) = multiply.left.as_index_and_b_options();
        let (c, c_options) = multiply.right.as_index_and_c_options();

        Instruction {
            operation: Operation::MULTIPLY,
            options: a_options | b_options | c_options,
            a,
            b,
            c,
        }
    }
}
