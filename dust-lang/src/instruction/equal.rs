use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, MemoryKind, OperandType, Operation};

pub struct Equal {
    pub comparator: bool,
    pub operand_type: OperandType,
    pub left_memory: MemoryKind,
    pub left_index: u16,
    pub right_memory: MemoryKind,
    pub right_index: u16,
}

impl From<&Instruction> for Equal {
    fn from(instruction: &Instruction) -> Self {
        Equal {
            comparator: instruction.a_field() != 0,
            operand_type: instruction.operand_type(),
            left_memory: instruction.b_memory(),
            left_index: instruction.b_field(),
            right_memory: instruction.c_memory(),
            right_index: instruction.c_field(),
        }
    }
}

impl From<Equal> for Instruction {
    fn from(equal: Equal) -> Self {
        let Equal {
            comparator,
            operand_type,
            left_memory,
            left_index,
            right_memory,
            right_index,
        } = equal;

        InstructionBuilder::new(Operation::EQUAL)
            .a_field(comparator as u16)
            .operand_type(operand_type)
            .b_memory(left_memory)
            .b_field(left_index)
            .c_memory(right_memory)
            .c_field(right_index)
            .build()
    }
}

impl Display for Equal {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Equal {
            comparator,
            operand_type: _,
            left_memory,
            left_index,
            right_memory,
            right_index,
        } = *self;
        let operator = if comparator { "==" } else { "≠" };

        write!(
            f,
            "if {left_memory}_{left_index} {operator} {right_memory}_{right_index} {{ jump +1 }}"
        )
    }
}
