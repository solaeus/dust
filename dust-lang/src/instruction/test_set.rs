use crate::{Argument, Instruction, Operation};

pub struct TestSet {
    pub destination: u8,
    pub argument: Argument,
    pub test_value: bool,
}

impl From<&Instruction> for TestSet {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let argument = instruction.b_as_argument();
        let test_value = instruction.c_field() != 0;

        TestSet {
            destination,
            argument,
            test_value,
        }
    }
}

impl From<TestSet> for Instruction {
    fn from(test_set: TestSet) -> Self {
        let operation = Operation::TEST;
        let a = test_set.destination;
        let (b, b_is_constant) = test_set.argument.as_index_and_constant_flag();
        let c = test_set.test_value as u8;

        Instruction::new(operation, a, b, c, b_is_constant, false, false)
    }
}
