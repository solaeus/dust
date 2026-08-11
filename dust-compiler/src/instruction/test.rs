use std::fmt::{self, Display, Formatter};

use crate::instruction::{Address, Instruction, InstructionBuilder, Operation};

pub struct Test {
    pub comparator: bool,
    pub operand: Address,
    pub jump_distance: u16,
}

impl From<Instruction> for Test {
    fn from(instruction: Instruction) -> Self {
        Test {
            comparator: instruction.a_field() != 0,
            operand: instruction.b_address(),
            jump_distance: instruction.c_field() as u16,
        }
    }
}

impl From<Test> for Instruction {
    fn from(test: Test) -> Self {
        let Test {
            comparator,
            operand,
            jump_distance,
        } = test;

        InstructionBuilder::new(Operation::TEST)
            .a_field(comparator as u16)
            .b_address(operand)
            .c_field(jump_distance)
            .build()
    }
}

impl Display for Test {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Test {
            comparator,
            operand,
            jump_distance,
        } = self;
        let bang = if *comparator { "" } else { "!" };

        write!(f, "if {bang}{operand} jump +{jump_distance}")
    }
}
