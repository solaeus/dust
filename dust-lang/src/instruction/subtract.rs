use crate::{Argument, Instruction, Operation};

pub struct Subtract {
    pub destination: u8,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Subtract {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a;
        let (left, right) = instruction.b_and_c_as_arguments();

        Subtract {
            destination,
            left,
            right,
        }
    }
}

impl From<Subtract> for Instruction {
    fn from(subtract: Subtract) -> Self {
        let a = subtract.destination;
        let (b, b_options) = subtract.left.as_index_and_b_options();
        let (c, c_options) = subtract.right.as_index_and_c_options();
        let metadata = Operation::Subtract as u8 | b_options.bits() | c_options.bits();

        Instruction { metadata, a, b, c }
    }
}
