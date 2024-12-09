use crate::{Argument, Instruction, Operation};

use super::InstructionOptions;

pub struct LessEqual {
    pub destination: u8,
    pub value: bool,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for LessEqual {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a;
        let value = instruction.d();
        let (left, right) = instruction.b_and_c_as_arguments();

        LessEqual {
            destination,
            value,
            left,
            right,
        }
    }
}

impl From<LessEqual> for Instruction {
    fn from(less_equal: LessEqual) -> Self {
        let a = less_equal.destination;
        let (b, b_options) = less_equal.left.as_index_and_b_options();
        let (c, c_options) = less_equal.right.as_index_and_c_options();
        let d_options = if less_equal.value {
            InstructionOptions::D_IS_TRUE
        } else {
            InstructionOptions::empty()
        };
        let metadata =
            Operation::LessEqual as u8 | b_options.bits() | c_options.bits() | d_options.bits();

        Instruction { metadata, a, b, c }
    }
}
