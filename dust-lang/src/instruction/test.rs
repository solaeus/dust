use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::InstructionFields;

pub struct Test {
    pub comparator: bool,
    pub operand_register: u16,
}

impl From<&Instruction> for Test {
    fn from(instruction: &Instruction) -> Self {
        let operand_register = instruction.a_field();
        let comparator = instruction.b_field() != 0;

        Test {
            operand_register,
            comparator,
        }
    }
}

impl From<Test> for Instruction {
    fn from(test: Test) -> Self {
        let a_field = test.operand_register;
        let c_field = test.comparator as u16;

        InstructionFields {
            operation: Operation::TEST,
            a_field,
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
            comparator,
        } = self;
        let bang = if *comparator { "" } else { "!" };

        write!(f, "if {bang}R_BOOL_{operand_register} {{ JUMP +1 }}")
    }
}
