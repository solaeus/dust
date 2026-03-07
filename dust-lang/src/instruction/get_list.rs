use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, MemoryKind, OperandType, Operation};

pub struct GetList {
    pub destination: u16,
    pub element_type: OperandType,
    pub list_index: u16,
    pub index_memory: MemoryKind,
    pub index_index: u16,
}

impl From<&Instruction> for GetList {
    fn from(instruction: &Instruction) -> Self {
        GetList {
            destination: instruction.a_field(),
            element_type: instruction.operand_type(),
            list_index: instruction.b_field(),
            index_memory: instruction.c_memory(),
            index_index: instruction.c_field(),
        }
    }
}

impl From<GetList> for Instruction {
    fn from(set_list: GetList) -> Self {
        let GetList {
            destination,
            element_type,
            list_index,
            index_memory,
            index_index,
        } = set_list;

        InstructionBuilder::new(Operation::GET_LIST)
            .a_field(destination)
            .b_field(list_index)
            .c_memory(index_memory)
            .c_field(index_index)
            .operand_type(element_type)
            .build()
    }
}

impl Display for GetList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let GetList {
            destination,
            element_type,
            list_index,
            index_memory,
            index_index,
        } = *self;

        write!(
            f,
            "reg_{destination}: {element_type} = reg_{list_index}[{index_memory}_{index_index}]"
        )
    }
}
