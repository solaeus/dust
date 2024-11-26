use crate::{Argument, Instruction, Operation};

pub struct TestSet {
    pub destination: u16,
    pub argument: Argument,
    pub value: bool,
}

impl From<&Instruction> for TestSet {
    fn from(instruction: &Instruction) -> Self {
        TestSet {
            destination: instruction.a(),
            argument: instruction.b_as_argument(),
            value: instruction.c_as_boolean(),
        }
    }
}

impl From<TestSet> for Instruction {
    fn from(test_set: TestSet) -> Self {
        *Instruction::new(Operation::TestSet)
            .set_a(test_set.destination)
            .set_b(test_set.argument.index())
            .set_b_is_constant(test_set.argument.is_constant())
            .set_b_is_local(test_set.argument.is_local())
            .set_c_to_boolean(test_set.value)
    }
}
