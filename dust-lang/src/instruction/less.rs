use crate::{Argument, Instruction, Operation};

pub struct Less {
    pub value: bool,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Less {
    fn from(instruction: &Instruction) -> Self {
        let (left, right) = instruction.b_and_c_as_arguments();

        Less {
            value: instruction.a_as_boolean(),
            left,
            right,
        }
    }
}

impl From<Less> for Instruction {
    fn from(less: Less) -> Self {
        *Instruction::new(Operation::Less)
            .set_a_to_boolean(less.value)
            .set_b(less.left.index())
            .set_b_is_constant(less.left.is_constant())
            .set_b_is_local(less.left.is_local())
            .set_c(less.right.index())
            .set_c_is_constant(less.right.is_constant())
            .set_c_is_local(less.right.is_local())
    }
}
