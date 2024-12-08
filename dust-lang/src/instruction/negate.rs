use crate::{Argument, Destination, Instruction, Operation};

pub struct Negate {
    pub destination: Destination,
    pub argument: Argument,
}

impl From<&Instruction> for Negate {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_as_destination();
        let argument = instruction.b_as_argument();

        Negate {
            destination,
            argument,
        }
    }
}

impl From<Negate> for Instruction {
    fn from(negate: Negate) -> Self {
        let (a, a_options) = negate.destination.as_index_and_a_options();
        let (b, b_options) = negate.argument.as_index_and_b_options();

        Instruction {
            operation: Operation::NEGATE,
            options: a_options | b_options,
            a,
            b,
            c: 0,
        }
    }
}
