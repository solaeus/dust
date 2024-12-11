use crate::{Argument, Instruction, Operation};

pub struct Divide {
    pub destination: u8,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Divide {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
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
        let operation = Operation::Divide;
        let a = divide.destination;
        let (b, b_is_constant) = divide.left.as_index_and_constant_flag();
        let (c, c_is_constant) = divide.right.as_index_and_constant_flag();

        Instruction::new(operation, a, b, c, b_is_constant, c_is_constant, false)
    }
}
