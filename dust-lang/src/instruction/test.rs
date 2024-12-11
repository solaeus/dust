use crate::{Argument, Instruction, Operation};

pub struct Test {
    pub argument: Argument,
    pub test_value: bool,
}

impl From<&Instruction> for Test {
    fn from(instruction: &Instruction) -> Self {
        let argument = instruction.b_as_argument();
        let test_value = instruction.c_field() != 0;

        Test {
            argument,
            test_value,
        }
    }
}

impl From<Test> for Instruction {
    fn from(test: Test) -> Self {
        let operation = Operation::Test;
        let (b, b_is_constant) = test.argument.as_index_and_constant_flag();
        let c = test.test_value as u8;

        Instruction::new(operation, 0, b, c, b_is_constant, false, false)
    }
}
