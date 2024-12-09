use crate::{Argument, Instruction, Operation};

pub struct TestSet {
    pub destination: u8,
    pub argument: Argument,
    pub test_value: bool,
}

impl From<&Instruction> for TestSet {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a;
        let argument = instruction.b_as_argument();
        let test_value = instruction.c != 0;

        TestSet {
            destination,
            argument,
            test_value,
        }
    }
}

impl From<TestSet> for Instruction {
    fn from(test_set: TestSet) -> Self {
        let a = test_set.destination;
        let (b, b_options) = test_set.argument.as_index_and_b_options();
        let c = test_set.test_value as u8;
        let metadata = Operation::TestSet as u8 | b_options.bits();

        Instruction { metadata, a, b, c }
    }
}
