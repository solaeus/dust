use crate::{Argument, Instruction, Operation};

pub struct Add {
    pub destination: u8,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Add {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
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
        let operation = Operation::ADD;
        let a = add.destination;
        let (b, b_is_constant) = add.left.as_index_and_constant_flag();
        let (c, c_is_constant) = add.right.as_index_and_constant_flag();

        Instruction::new(operation, a, b, c, b_is_constant, c_is_constant, false)
    }
}
