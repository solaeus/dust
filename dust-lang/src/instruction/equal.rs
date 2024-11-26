use crate::{Argument, Instruction, Operation};

pub struct Equal {
    pub value: bool,
    pub left: Argument,
    pub right: Argument,
}

impl From<&Instruction> for Equal {
    fn from(instruction: &Instruction) -> Self {
        let (left, right) = instruction.b_and_c_as_arguments();

        Equal {
            value: instruction.a_as_boolean(),
            left,
            right,
        }
    }
}

impl From<Equal> for Instruction {
    fn from(equal: Equal) -> Self {
        *Instruction::new(Operation::Equal)
            .set_a_to_boolean(equal.value)
            .set_b(equal.left.index())
            .set_b_is_constant(equal.left.is_constant())
            .set_b_is_local(equal.left.is_local())
            .set_c(equal.right.index())
            .set_c_is_constant(equal.right.is_constant())
            .set_c_is_local(equal.right.is_local())
    }
}
