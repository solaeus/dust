use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, Operation};

pub struct Test {
    pub comparator: bool,
    pub operand: Address,
}

impl From<Instruction> for Test {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.a_field() != 0;
        let operand = instruction.b_address();

        Test {
            comparator,
            operand,
        }
    }
}

impl From<Test> for Instruction {
    fn from(test: Test) -> Self {
        let a_field = test.comparator as u16;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = test.operand;

        InstructionFields {
            operation: Operation::TEST,
            a_field,
            b_field,
            b_memory_kind,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Test {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Test {
            operand,
            comparator,
        } = self;
        let bang = if *comparator { "" } else { "!" };

        write!(f, "if {bang}reg_{} jump +1", operand.index)
    }
}
