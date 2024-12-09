use crate::{Argument, Instruction, Operation};

pub struct Test {
    pub argument: Argument,
    pub test_value: bool,
}

impl From<&Instruction> for Test {
    fn from(instruction: &Instruction) -> Self {
        let argument = instruction.b_as_argument();
        let test_value = instruction.c != 0;

        Test {
            argument,
            test_value,
        }
    }
}

impl From<Test> for Instruction {
    fn from(test: Test) -> Self {
        let a = 0;
        let (b, options) = test.argument.as_index_and_b_options();
        let c = test.test_value as u8;
        let metadata = Operation::Test as u8 | options.bits();

        Instruction { metadata, a, b, c }
    }
}
