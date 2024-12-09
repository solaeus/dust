use crate::{Argument, Instruction, Operation};

pub struct Negate {
    pub destination: u8,
    pub argument: Argument,
}

impl From<&Instruction> for Negate {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a;
        let argument = instruction.b_as_argument();

        Negate {
            destination,
            argument,
        }
    }
}

impl From<Negate> for Instruction {
    fn from(negate: Negate) -> Self {
        let a = negate.destination;
        let (b, b_options) = negate.argument.as_index_and_b_options();
        let c = 0;
        let metadata = Operation::Negate as u8 | b_options.bits();

        Instruction { metadata, a, b, c }
    }
}
