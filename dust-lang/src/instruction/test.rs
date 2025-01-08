use crate::{Instruction, Operation};

pub struct Test {
    pub operand_register: u8,
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
        Instruction::new(
            Operation::TEST,
            0,
            test.operand_register,
            test.test_value as u8,
            false,
            false,
            false,
        )
    }
}
