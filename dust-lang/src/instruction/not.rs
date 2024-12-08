use crate::{Argument, Destination, Instruction, Operation};

pub struct Not {
    pub destination: Destination,
    pub argument: Argument,
}

impl From<&Instruction> for Not {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_as_destination();
        let argument = instruction.b_as_argument();

        Not {
            destination,
            argument,
        }
    }
}

impl From<Not> for Instruction {
    fn from(not: Not) -> Self {
        let (a, a_options) = not.destination.as_index_and_a_options();
        let (b, b_options) = not.argument.as_index_and_b_options();

        Instruction {
            operation: Operation::NOT,
            options: a_options | b_options,
            a,
            b,
            c: 0,
        }
    }
}
