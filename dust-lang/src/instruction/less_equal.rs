use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, MemoryKind, OperandType, Operation};

pub struct LessEqual {
    pub comparator: bool,
    pub operand_type: OperandType,
    pub left_memory: MemoryKind,
    pub left_index: u16,
    pub right_memory: MemoryKind,
    pub right_index: u16,
}

impl From<&Instruction> for LessEqual {
    fn from(instruction: &Instruction) -> Self {
        LessEqual {
            comparator: instruction.a_field() != 0,
            operand_type: instruction.operand_type(),
            left_memory: instruction.b_memory(),
            left_index: instruction.b_field(),
            right_memory: instruction.c_memory(),
            right_index: instruction.c_field(),
        }
    }
}

impl From<LessEqual> for Instruction {
    fn from(less_equal: LessEqual) -> Self {
        let LessEqual {
            comparator,
            operand_type,
            left_memory,
            left_index,
            right_memory,
            right_index,
        } = less_equal;

        InstructionBuilder::new(Operation::LESS_EQUAL)
            .a_field(if comparator { 1 } else { 0 })
            .operand_type(operand_type)
            .b_memory(left_memory)
            .b_field(left_index)
            .c_memory(right_memory)
            .c_field(right_index)
            .build()
    }
}

impl Display for LessEqual {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let LessEqual {
            comparator,
            operand_type,
            left_memory,
            left_index,
            right_memory,
            right_index,
        } = *self;
        let operator = if comparator { "≤" } else { ">" };
        let left_memory = left_memory.as_string(operand_type);
        let right_memory = right_memory.as_string(operand_type);

        write!(
            f,
            "if {left_memory}_{left_index} {operator} {right_memory}_{right_index} {{ jump +1 }}"
        )
    }
}
