use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionBuilder, Operand, Operation};

pub struct SubtractFloat {
    pub destination: u16,
    pub left: Operand,
    pub right: Operand,
}

impl From<Instruction> for SubtractFloat {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.a_field();
        let (left, right) = instruction.b_and_c_as_operands();

        SubtractFloat {
            destination,
            left,
            right,
        }
    }
}

impl From<SubtractFloat> for Instruction {
    fn from(subtract_float: SubtractFloat) -> Self {
        let operation = Operation::SUBTRACT_FLOAT;
        let a_field = subtract_float.destination;
        let (b_field, b_is_constant) = subtract_float.left.as_index_and_constant_flag();
        let (c_field, c_is_constant) = subtract_float.right.as_index_and_constant_flag();

        InstructionBuilder {
            operation,
            a_field,
            b_field,
            c_field,
            b_is_constant,
            c_is_constant,
            ..Default::default()
        }
        .build()
    }
}

impl Display for SubtractFloat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let SubtractFloat {
            destination,
            left,
            right,
        } = self;

        write!(f, "R{} = {} - {}", destination, left, right)
    }
}
