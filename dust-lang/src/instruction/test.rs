use std::fmt::{self, Display, Formatter};

use crate::instruction::OperandType;

use super::{Address, Instruction, InstructionFields, Operation};

pub struct Test {
    pub comparator: bool,
    pub operand: Address,
    pub jump_distance: u16,
}

impl From<Instruction> for Test {
    fn from(instruction: Instruction) -> Self {
        let comparator = instruction.a_field() != 0;
        let operand = instruction.b_address();
        let jump_distance = instruction.c_field();

        Test {
            comparator,
            operand,
            jump_distance,
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
        let c_field = test.jump_distance;

        InstructionFields {
            operation: Operation::TEST,
            a_field,
            b_field,
            b_memory_kind,
            c_field,
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
            jump_distance,
        } = self;
        let bang = if *comparator { "" } else { "!" };

        write!(f, "if {bang}")?;
        operand.display(f, OperandType::BOOLEAN)?;
        write!(f, " {{ jump +{jump_distance} }}")
    }
}
