use crate::{Instruction, Operation};

use super::InstructionOptions;

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
        let b = r#return.should_return_value as u16;

        Instruction {
            operation: Operation::RETURN,
            options: InstructionOptions::empty(),
            a: 0,
            b,
            c: 0,
        }
    }
}
