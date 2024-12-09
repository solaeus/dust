use crate::{Argument, Instruction, Operation};

use super::InstructionOptions;

pub struct Equal {
    pub destination: u8,
    pub value: bool,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Equal {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a;
        let value = instruction.d();
        let (left, right) = instruction.b_and_c_as_arguments();

        Equal {
            destination,
            value,
            left,
            right,
        }
    }
}

impl From<Equal> for Instruction {
    fn from(equal: Equal) -> Self {
        let a = equal.destination;
        let (b, b_options) = equal.left.as_index_and_b_options();
        let (c, c_options) = equal.right.as_index_and_c_options();
        let d_options = if equal.value {
            InstructionOptions::D_IS_TRUE
        } else {
            InstructionOptions::empty()
        };
        let metadata =
            Operation::Equal as u8 | b_options.bits() | c_options.bits() | d_options.bits();

        Instruction { metadata, a, b, c }
    }
}
