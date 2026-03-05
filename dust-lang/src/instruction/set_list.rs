use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, MemoryKind, OperandType, Operation};

pub struct SetList {
    pub destination_list: u16,
    pub element_type: OperandType,
    pub source_memory: MemoryKind,
    pub source_index: u16,
    pub index_memory: MemoryKind,
    pub index_index: u16,
}

impl From<&Instruction> for SetList {
    fn from(instruction: &Instruction) -> Self {
        SetList {
            destination_list: instruction.a_field(),
            element_type: instruction.operand_type(),
            source_memory: instruction.b_memory(),
            source_index: instruction.b_field(),
            index_memory: instruction.c_memory(),
            index_index: instruction.c_field(),
        }
    }
}

impl From<SetList> for Instruction {
    fn from(set_list: SetList) -> Self {
        let SetList {
            destination_list,
            element_type,
            source_memory,
            source_index,
            index_memory,
            index_index,
        } = set_list;

        InstructionBuilder::new(Operation::SET_LIST)
            .a_field(destination_list)
            .b_memory(source_memory)
            .b_field(source_index)
            .c_memory(index_memory)
            .c_field(index_index)
            .operand_type(element_type)
            .build()
    }
}

impl Display for SetList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let SetList {
            destination_list,
            element_type,
            source_memory,
            source_index,
            index_memory,
            index_index,
        } = *self;
        let source_memory = source_memory.as_string(element_type);
        let index_memory = index_memory.as_string(OperandType::U_64);

        write!(
            f,
            "reg_{destination_list}[{index_memory}_{index_index}] = {source_memory}_{source_index}"
        )
    }
}
