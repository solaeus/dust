use crate::{Argument, Instruction, Operation};

pub struct Multiply {
    pub destination: u8,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Multiply {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a;
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
        let a = multiply.destination;
        let (b, b_options) = multiply.left.as_index_and_b_options();
        let (c, c_options) = multiply.right.as_index_and_c_options();
        let metadata = Operation::Multiply as u8 | b_options.bits() | c_options.bits();

        Instruction { metadata, a, b, c }
    }
}
