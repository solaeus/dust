use crate::{Instruction, Operation};

use super::InstructionBuilder;

pub struct Test {
    pub operand_register: u16,
    pub test_value: bool,
}

impl From<Instruction> for Test {
    fn from(instruction: Instruction) -> Self {
        let operand_register = instruction.b_field();
        let test_value = instruction.c_field() != 0;

        Test {
            operand_register,
            test_value,
        }
    }
}

impl From<Test> for Instruction {
    fn from(test: Test) -> Self {
        let b_field = test.operand_register;
        let c_field = test.test_value as u16;

        InstructionBuilder {
            operation: Operation::TEST,
            b_field,
            c_field,
            ..Default::default()
        }
        .build()
    }
}
