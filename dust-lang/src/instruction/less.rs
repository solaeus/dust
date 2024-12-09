use crate::{Argument, Instruction, Operation};

use super::InstructionOptions;

pub struct Less {
    pub destination: u8,
    pub value: bool,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Less {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a;
        let value = instruction.d();
        let (left, right) = instruction.b_and_c_as_arguments();

        Less {
            destination,
            value,
            left,
            right,
        }
    }
}

impl From<Less> for Instruction {
    fn from(less: Less) -> Self {
        let a = less.destination;
        let (b, b_options) = less.left.as_index_and_b_options();
        let (c, c_options) = less.right.as_index_and_c_options();
        let d_options = if less.value {
            InstructionOptions::D_IS_TRUE
        } else {
            InstructionOptions::empty()
        };
        let metadata =
            Operation::Less as u8 | b_options.bits() | c_options.bits() | d_options.bits();

        Instruction { metadata, a, b, c }
    }
}
