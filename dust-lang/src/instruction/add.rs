use crate::{Argument, Instruction, Operation};

pub struct Add {
    pub destination: u8,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Add {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a;
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
        let operation = Operation::Add;
        let a = add.destination;
        let (b, b_options) = add.left.as_index_and_b_options();
        let (c, c_options) = add.right.as_index_and_c_options();
        let metadata = operation as u8 | b_options.bits() | c_options.bits();

        Instruction { metadata, a, b, c }
    }
}
