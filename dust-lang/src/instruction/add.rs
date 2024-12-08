use crate::{Argument, Destination, Instruction, Operation};

pub struct Add {
    pub destination: Destination,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Add {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_as_destination();
        let (left, right) = instruction.b_and_c_as_arguments();

        Add {
            destination,
            left,
            right,
        }
    }
}

impl From<Add> for Instruction {
    fn from(add: Add) -> Self {
        let (a, a_options) = add.destination.as_index_and_a_options();
        let (b, b_options) = add.left.as_index_and_b_options();
        let (c, c_options) = add.right.as_index_and_c_options();

        Instruction {
            operation: Operation::ADD,
            options: a_options | b_options | c_options,
            a,
            b,
            c,
        }
    }
}
