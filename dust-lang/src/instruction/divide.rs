use crate::{Argument, Instruction, Operation};

pub struct Divide {
    pub destination: u8,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Divide {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a;
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
        let a = divide.destination;
        let (b, b_options) = divide.left.as_index_and_b_options();
        let (c, c_options) = divide.right.as_index_and_c_options();
        let metadata = Operation::Divide as u8 | b_options.bits() | c_options.bits();

        Instruction { metadata, a, b, c }
    }
}
