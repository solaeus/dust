use crate::{Instruction, Operation};

pub struct Return {
    pub should_return_value: bool,
    pub return_register: u8,
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
        let b = r#return.should_return_value as u8;
        let c = r#return.return_register;

        Instruction::new(operation, 0, b, c, false, false, false)
    }
}
