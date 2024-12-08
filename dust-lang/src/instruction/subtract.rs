use crate::{Argument, Destination, Instruction, Operation};

pub struct Subtract {
    pub destination: Destination,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Subtract {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_as_destination();
        let (left, right) = instruction.b_and_c_as_arguments();

        Subtract {
            destination,
            left,
            right,
        }
    }
}

impl From<Subtract> for Instruction {
    fn from(subtract: Subtract) -> Self {
        let (a, a_options) = subtract.destination.as_index_and_a_options();
        let (b, b_options) = subtract.left.as_index_and_b_options();
        let (c, c_options) = subtract.right.as_index_and_c_options();

        Instruction {
            operation: Operation::SUBTRACT,
            options: a_options | b_options | c_options,
            a,
            b,
            c,
        }
    }
}
