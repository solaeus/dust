use crate::{Instruction, Operation};

pub struct Return {
    pub should_return_value: bool,
}

impl From<&Instruction> for Return {
    fn from(instruction: &Instruction) -> Self {
        Return {
            should_return_value: instruction.b_as_boolean(),
        }
    }
}

impl From<Return> for Instruction {
    fn from(r#return: Return) -> Self {
        *Instruction::new(Operation::Return).set_b_to_boolean(r#return.should_return_value)
    }
}
