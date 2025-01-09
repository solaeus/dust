use crate::{Argument, Instruction, Operation};

pub struct Less {
    pub value: bool,
    pub left: Argument,
    pub right: Argument,
}

impl From<Instruction> for Less {
    fn from(instruction: Instruction) -> Self {
        let value = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_arguments();

        Less { value, left, right }
    }
}

impl From<Less> for Instruction {
    fn from(less: Less) -> Self {
        let operation = Operation::LESS;
        let (b, b_is_constant) = less.left.as_index_and_constant_flag();
        let (c, c_is_constant) = less.right.as_index_and_constant_flag();
        let d = less.value;

        Instruction::new(operation, 0, b, c, b_is_constant, c_is_constant, d)
    }
}
