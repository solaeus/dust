use crate::{Argument, Destination, Instruction, Operation};

use super::InstructionOptions;

pub struct LessEqual {
    pub destination: Destination,
    pub value: bool,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for LessEqual {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_as_destination();
        let value = instruction.options.d();
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
        let (a, a_options) = less_equal.destination.as_index_and_a_options();
        let (b, b_options) = less_equal.left.as_index_and_b_options();
        let (c, c_options) = less_equal.right.as_index_and_c_options();
        let d_options = if less_equal.value {
            InstructionOptions::D_IS_TRUE
        } else {
            InstructionOptions::empty()
        };

        Instruction {
            operation: Operation::LESS_EQUAL,
            options: a_options | b_options | c_options | d_options,
            a,
            b,
            c,
        }
    }
}
