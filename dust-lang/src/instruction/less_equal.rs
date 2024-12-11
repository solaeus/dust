use crate::{Argument, Instruction, Operation};

pub struct LessEqual {
    pub destination: u8,
    pub value: bool,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for LessEqual {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let value = instruction.d_field();
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
        let operation = Operation::LESS_EQUAL;
        let a = less_equal.destination;
        let (b, b_options) = less_equal.left.as_index_and_constant_flag();
        let (c, c_options) = less_equal.right.as_index_and_constant_flag();
        let d = less_equal.value;

        Instruction::new(operation, a, b, c, b_options, c_options, d)
    }
}
