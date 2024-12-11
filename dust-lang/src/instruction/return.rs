use crate::{Instruction, Operation};

pub struct Return {
    pub should_return_value: bool,
}

impl From<&Instruction> for Return {
    fn from(instruction: &Instruction) -> Self {
        let should_return_value = instruction.b_field() != 0;

        Return {
            should_return_value,
        }
    }
}

impl From<Return> for Instruction {
    fn from(r#return: Return) -> Self {
        let operation = Operation::RETURN;
        let b = r#return.should_return_value as u8;

        Instruction::new(operation, 0, b, 0, false, false, false)
    }
}
