use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::InstructionFields;

pub struct Test {
    pub operand_register: u16,
    pub test_value: bool,
}

impl From<&Instruction> for Test {
    fn from(instruction: &Instruction) -> Self {
        let operand_register = instruction.b_field();
        let test_value = instruction.c_field() != 0;

        Test {
            operand_register,
            test_value,
        }
    }
}

impl From<Test> for Instruction {
    fn from(test: Test) -> Self {
        let b_field = test.operand_register;
        let c_field = test.test_value as u16;

        InstructionFields {
            operation: Operation::TEST,
            b_field,
            c_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Test {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Test {
            operand_register,
            test_value,
        } = self;
        let bang = if *test_value { "" } else { "!" };

        write!(f, "if {bang}R_BOOL_{operand_register} {{ JUMP +1 }}")
    }
}
