use crate::{Argument, Instruction, Operation};

pub struct LessEqual {
    pub value: bool,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for LessEqual {
    fn from(instruction: &Instruction) -> Self {
        let (left, right) = instruction.b_and_c_as_arguments();

        LessEqual {
            value: instruction.a_as_boolean(),
            left,
            right,
        }
    }
}

impl From<LessEqual> for Instruction {
    fn from(less_equal: LessEqual) -> Self {
        *Instruction::new(Operation::LessEqual)
            .set_a_to_boolean(less_equal.value)
            .set_b(less_equal.left.index())
            .set_b_is_constant(less_equal.left.is_constant())
            .set_b_is_local(less_equal.left.is_local())
            .set_c(less_equal.right.index())
            .set_c_is_constant(less_equal.right.is_constant())
            .set_c_is_local(less_equal.right.is_local())
    }
}
