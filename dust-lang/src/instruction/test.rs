use crate::{Argument, Instruction, Operation};

pub struct Test {
    pub argument: Argument,
    pub value: bool,
}

impl From<&Instruction> for Test {
    fn from(instruction: &Instruction) -> Self {
        Test {
            argument: instruction.b_as_argument(),
            value: instruction.c_as_boolean(),
        }
    }
}

impl From<Test> for Instruction {
    fn from(test: Test) -> Self {
        *Instruction::new(Operation::Test)
            .set_b(test.argument.index())
            .set_b_is_constant(test.argument.is_constant())
            .set_b_is_local(test.argument.is_local())
            .set_c_to_boolean(test.value)
    }
}
