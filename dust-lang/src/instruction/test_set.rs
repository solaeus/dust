use crate::{Argument, Destination, Instruction, Operation};

pub struct TestSet {
    pub destination: Destination,
    pub argument: Argument,
    pub test_value: bool,
}

impl From<&Instruction> for TestSet {
    fn from(instruction: &Instruction) -> Self {
        let destination = if instruction.a_is_local() {
            Destination::Local(instruction.a())
        } else {
            Destination::Register(instruction.a())
        };

        TestSet {
            destination,
            argument: instruction.b_as_argument(),
            test_value: instruction.c_as_boolean(),
        }
    }
}

impl From<TestSet> for Instruction {
    fn from(test_set: TestSet) -> Self {
        let (a, a_is_local) = match test_set.destination {
            Destination::Local(local) => (local, true),
            Destination::Register(register) => (register, false),
        };

        *Instruction::new(Operation::TestSet)
            .set_a(a)
            .set_a_is_local(a_is_local)
            .set_b(test_set.argument.index())
            .set_b_is_constant(test_set.argument.is_constant())
            .set_b_is_local(test_set.argument.is_local())
            .set_c_to_boolean(test_set.test_value)
    }
}
