use crate::{Argument, Destination, Instruction, Operation};

pub struct TestSet {
    pub destination: Destination,
    pub argument: Argument,
    pub test_value: bool,
}

impl From<&Instruction> for TestSet {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_as_destination();
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
        let (a, a_options) = test_set.destination.as_index_and_a_options();
        let (b, b_options) = test_set.argument.as_index_and_b_options();
        let c = test_set.test_value as u16;

        Instruction {
            operation: Operation::TEST_SET,
            options: a_options | b_options,
            a,
            b,
            c,
        }
    }
}
