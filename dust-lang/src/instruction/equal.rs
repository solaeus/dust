use crate::{Argument, Instruction, Operation};

pub struct Equal {
    pub value: bool,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Equal {
    fn from(instruction: &Instruction) -> Self {
        let value = instruction.d_field();
        let (left, right) = instruction.b_and_c_as_arguments();

        Equal { value, left, right }
    }
}

impl From<Equal> for Instruction {
    fn from(equal: Equal) -> Self {
        let operation = Operation::EQUAL;
        let (b, b_is_constant) = equal.left.as_index_and_constant_flag();
        let (c, c_is_constant) = equal.right.as_index_and_constant_flag();
        let d = equal.value;

        Instruction::new(operation, 0, b, c, b_is_constant, c_is_constant, d)
    }
}
