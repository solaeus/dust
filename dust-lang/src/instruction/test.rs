use std::fmt::{self, Display, Formatter};

use crate::instruction::OperandType;

use super::{Instruction, InstructionBuilder, MemoryKind, Operation};

pub struct Test {
    pub comparator: bool,
    pub operand_memory: MemoryKind,
    pub operand_index: u16,
    pub jump_distance: u16,
}

impl From<&Instruction> for Test {
    fn from(instruction: &Instruction) -> Self {
        Test {
            comparator: instruction.a_field() != 0,
            operand_memory: instruction.b_memory(),
            operand_index: instruction.b_field(),
            jump_distance: instruction.c_field(),
        }
    }
}

impl From<Test> for Instruction {
    fn from(test: Test) -> Self {
        let Test {
            comparator,
            operand_memory,
            operand_index,
            jump_distance,
        } = test;

        InstructionBuilder::new(Operation::TEST)
            .a_field(comparator as u16)
            .b_memory(operand_memory)
            .b_field(operand_index)
            .c_field(jump_distance)
            .build()
    }
}

impl Display for Test {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Test {
            comparator,
            operand_memory,
            operand_index,
            jump_distance,
        } = self;
        let bang = if *comparator { "" } else { "!" };
        let operand_memory = operand_memory.as_string(OperandType::BOOLEAN);

        write!(
            f,
            "if {bang}{operand_memory}_{operand_index} {{ jump +{jump_distance} }}"
        )
    }
}
