use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, MemoryKind, OperandType, Operation};

pub struct Add {
    pub destination: u16,
    pub operand_type: OperandType,
    pub left_memory: MemoryKind,
    pub left_index: u16,
    pub right_memory: MemoryKind,
    pub right_index: u16,
}

impl From<&Instruction> for Add {
    fn from(instruction: &Instruction) -> Self {
        Add {
            destination: instruction.a_field(),
            operand_type: instruction.operand_type(),
            left_memory: instruction.b_memory(),
            left_index: instruction.b_field(),
            right_memory: instruction.c_memory(),
            right_index: instruction.c_field(),
        }
    }
}

impl From<Add> for Instruction {
    fn from(add: Add) -> Self {
        let Add {
            destination,
            operand_type,
            left_memory,
            left_index,
            right_memory,
            right_index,
        } = add;

        InstructionBuilder::new(Operation::ADD)
            .a_field(destination)
            .b_memory(left_memory)
            .b_field(left_index)
            .c_memory(right_memory)
            .c_field(right_index)
            .operand_type(operand_type)
            .build()
    }
}

impl Display for Add {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Add {
            destination,
            operand_type,
            left_index,
            left_memory,
            right_index,
            right_memory,
        } = *self;
        let sum_type = if matches!(
            operand_type,
            OperandType::CHARACTER_STRING | OperandType::STRING_CHARACTER
        ) {
            OperandType::STRING
        } else {
            operand_type
        };

        write!(
            f,
            "reg_{destination}: {sum_type} = {left_memory}_{left_index} + {right_memory}_{right_index}"
        )
    }
}
