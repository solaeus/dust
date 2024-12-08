use crate::{Argument, Destination, Instruction, Operation};

use super::InstructionOptions;

pub struct Less {
    pub destination: Destination,
    pub value: bool,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Less {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_as_destination();
        let value = instruction.options.d();
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
        let (a, a_options) = less.destination.as_index_and_a_options();
        let (b, b_options) = less.left.as_index_and_b_options();
        let (c, c_options) = less.right.as_index_and_c_options();
        let d_options = if less.value {
            InstructionOptions::D_IS_TRUE
        } else {
            InstructionOptions::empty()
        };

        Instruction {
            operation: Operation::LESS,
            options: a_options | b_options | c_options | d_options,
            a,
            b,
            c,
        }
    }
}
