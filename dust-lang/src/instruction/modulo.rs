use crate::{Argument, Instruction, Operation};

pub struct Modulo {
    pub destination: u8,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Modulo {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a;
        let (left, right) = instruction.b_and_c_as_arguments();

        Modulo {
            destination,
            left,
            right,
        }
    }
}

impl From<Modulo> for Instruction {
    fn from(modulo: Modulo) -> Self {
        let a = modulo.destination;
        let (b, b_options) = modulo.left.as_index_and_b_options();
        let (c, c_options) = modulo.right.as_index_and_c_options();
        let metadata = Operation::Modulo as u8 | b_options.bits() | c_options.bits();

        Instruction { metadata, a, b, c }
    }
}
