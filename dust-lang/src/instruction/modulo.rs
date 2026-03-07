use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, MemoryKind, OperandType, Operation};

pub struct Modulo {
    pub destination: u16,
    pub operand_type: OperandType,
    pub left_memory: MemoryKind,
    pub left_index: u16,
    pub right_memory: MemoryKind,
    pub right_index: u16,
}

impl From<&Instruction> for Modulo {
    fn from(instruction: &Instruction) -> Self {
        Modulo {
            destination: instruction.a_field(),
            operand_type: instruction.operand_type(),
            left_memory: instruction.b_memory(),
            left_index: instruction.b_field(),
            right_memory: instruction.c_memory(),
            right_index: instruction.c_field(),
        }
    }
}

impl From<Modulo> for Instruction {
    fn from(modulo: Modulo) -> Self {
        let Modulo {
            destination,
            operand_type,
            left_memory,
            left_index,
            right_memory,
            right_index,
        } = modulo;

        InstructionBuilder::new(Operation::MODULO)
            .a_field(destination)
            .operand_type(operand_type)
            .b_memory(left_memory)
            .b_field(left_index)
            .c_memory(right_memory)
            .c_field(right_index)
            .build()
    }
}

impl Display for Modulo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Modulo {
            destination,
            operand_type,
            left_memory,
            left_index,
            right_memory,
            right_index,
        } = *self;

        write!(
            f,
            "reg_{destination}: {operand_type} = {left_memory}_{left_index} % {right_memory}_{right_index}"
        )
    }
}
