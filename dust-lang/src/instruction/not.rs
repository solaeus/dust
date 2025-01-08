use crate::{Argument, Instruction, Operation};

pub struct Not {
    pub destination: u8,
    pub argument: Argument,
}

impl From<Instruction> for Not {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let argument = instruction.b_as_argument();

        Not {
            destination,
            argument,
        }
    }
}

impl From<Not> for Instruction {
    fn from(not: Not) -> Self {
        let operation = Operation::NOT;
        let a = not.destination;
        let (b, b_is_constant) = not.argument.as_index_and_constant_flag();

        Instruction::new(operation, a, b, 0, b_is_constant, false, false)
    }
}
