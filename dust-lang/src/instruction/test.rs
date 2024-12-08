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
        let (b, options) = test.argument.as_index_and_b_options();
        let c = test.test_value as u16;

        Instruction {
            operation: Operation::TEST,
            options,
            a: 0,
            b,
            c,
        }
    }
}
