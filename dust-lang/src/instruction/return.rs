use crate::{Instruction, Operation};

pub struct Return {
    pub should_return_value: bool,
}

impl From<&Instruction> for Return {
    fn from(instruction: &Instruction) -> Self {
        let should_return_value = instruction.b != 0;

        Return {
            should_return_value,
        }
    }
}

impl From<Return> for Instruction {
    fn from(r#return: Return) -> Self {
        let metadata = Operation::Return as u8;
        let a = 0;
        let b = r#return.should_return_value as u8;
        let c = 0;

        Instruction { metadata, a, b, c }
    }
}
