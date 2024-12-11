use crate::{Argument, Instruction, Operation};

pub struct Multiply {
    pub destination: u8,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Multiply {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_arguments();

        Multiply {
            destination,
            left,
            right,
        }
    }
}

impl From<Multiply> for Instruction {
    fn from(multiply: Multiply) -> Self {
        let operation = Operation::Multiply;
        let a = multiply.destination;
        let (b, b_options) = multiply.left.as_index_and_constant_flag();
        let (c, c_options) = multiply.right.as_index_and_constant_flag();

        Instruction::new(operation, a, b, c, b_options, c_options, false)
    }
}
