use crate::{Instruction, Operation};

use super::InstructionBuilder;

pub struct Return {
    pub should_return_value: bool,
    pub return_register: u16,
}

impl From<Instruction> for Return {
    fn from(instruction: Instruction) -> Self {
        let should_return_value = instruction.b_field() != 0;
        let return_register = instruction.c_field();

        Return {
            should_return_value,
            return_register,
        }
    }
}

impl From<Return> for Instruction {
    fn from(r#return: Return) -> Self {
        let operation = Operation::RETURN;
        let b_field = r#return.should_return_value as u16;
        let c_field = r#return.return_register;

        InstructionBuilder {
            operation,
            b_field,
            c_field,
            ..Default::default()
        }
        .build()
    }
}
