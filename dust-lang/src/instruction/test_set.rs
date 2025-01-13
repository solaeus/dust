use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operand, Operation};

use super::InstructionBuilder;

pub struct TestSet {
    pub destination: u16,
    pub argument: Operand,
    pub test_value: bool,
}

impl From<Instruction> for TestSet {
    fn from(instruction: Instruction) -> Self {
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
        let a_field = test_set.destination;
        let (b_field, b_is_constant) = test_set.argument.as_index_and_constant_flag();
        let c_field = test_set.test_value as u16;

        InstructionBuilder {
            operation,
            a_field,
            b_field,
            c_field,
            b_is_constant,
            ..Default::default()
        }
        .build()
    }
}

impl Display for TestSet {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let TestSet {
            destination,
            argument,
            test_value,
        } = self;
        let bang = if *test_value { "" } else { "!" };

        write!(
            f,
            "if {bang}{argument} {{ JUMP +1 }} else {{ R{destination} = {argument} }}"
        )
    }
}
