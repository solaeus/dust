use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, MemoryKind, OperandType, Operation};

pub struct SetIndex {
    pub base_register: u16,
    pub operand_type: OperandType,
    pub index_memory: MemoryKind,
    pub index_index: u16,
    pub source_memory: MemoryKind,
    pub source_index: u16,
}

impl From<&Instruction> for SetIndex {
    fn from(instruction: &Instruction) -> Self {
        SetIndex {
            base_register: instruction.a_field(),
            operand_type: instruction.operand_type(),
            index_memory: instruction.b_memory(),
            index_index: instruction.b_field(),
            source_memory: instruction.c_memory(),
            source_index: instruction.c_field(),
        }
    }
}

impl From<SetIndex> for Instruction {
    fn from(set_index: SetIndex) -> Self {
        let SetIndex {
            base_register,
            operand_type,
            index_memory,
            index_index,
            source_memory,
            source_index,
        } = set_index;

        InstructionBuilder::new(Operation::SET_INDEX)
            .a_field(base_register)
            .operand_type(operand_type)
            .b_memory(index_memory)
            .b_field(index_index)
            .c_memory(source_memory)
            .c_field(source_index)
            .build()
    }
}

impl Display for SetIndex {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let SetIndex {
            base_register,
            operand_type,
            index_memory,
            index_index,
            source_memory,
            source_index,
        } = *self;

        write!(
            f,
            "reg_{base_register}[{index_memory}_{index_index}]: {operand_type} = {source_memory}_{source_index}"
        )
    }
}
