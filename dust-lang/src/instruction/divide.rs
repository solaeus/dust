use crate::{Argument, Destination, Instruction, Operation};

pub struct Divide {
    pub destination: Destination,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Divide {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_as_destination();
        let (left, right) = instruction.b_and_c_as_arguments();

        Divide {
            destination,
            left,
            right,
        }
    }
}

impl From<Divide> for Instruction {
    fn from(divide: Divide) -> Self {
        let (a, a_options) = divide.destination.as_index_and_a_options();
        let (b, b_options) = divide.left.as_index_and_b_options();
        let (c, c_options) = divide.right.as_index_and_c_options();

        Instruction {
            operation: Operation::DIVIDE,
            options: a_options | b_options | c_options,
            a,
            b,
            c,
        }
    }
}
