use crate::{Argument, Instruction, Operation};

pub struct Negate {
    pub destination: u8,
    pub argument: Argument,
}

impl From<&Instruction> for Negate {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let argument = instruction.b_as_argument();

        Negate {
            destination,
            argument,
        }
    }
}

impl From<Negate> for Instruction {
    fn from(negate: Negate) -> Self {
        let operation = Operation::Negate;
        let a = negate.destination;
        let (b, b_is_constant) = negate.argument.as_index_and_constant_flag();
        let c = 0;

        Instruction::new(operation, a, b, c, b_is_constant, false, false)
    }
}
