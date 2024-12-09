use crate::{Argument, Instruction, Operation};

pub struct Not {
    pub destination: u8,
    pub argument: Argument,
}

impl From<&Instruction> for Not {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a;
        let argument = instruction.b_as_argument();

        Not {
            destination,
            argument,
        }
    }
}

impl From<Not> for Instruction {
    fn from(not: Not) -> Self {
        let a = not.destination;
        let (b, b_options) = not.argument.as_index_and_b_options();
        let c = 0;
        let metadata = Operation::Not as u8 | b_options.bits();

        Instruction { metadata, a, b, c }
    }
}
