use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, MemoryKind, OperandType, Operation};

pub struct NewList {
    pub destination: u16,
    pub element_type: OperandType,
    pub length_memory: MemoryKind,
    pub length_index: u16,
    pub element_size: u16,
}

impl From<&Instruction> for NewList {
    fn from(instruction: &Instruction) -> Self {
        NewList {
            destination: instruction.a_field(),
            element_type: instruction.operand_type(),
            length_memory: instruction.b_memory(),
            length_index: instruction.b_field(),
            element_size: instruction.c_field(),
        }
    }
}

impl From<NewList> for Instruction {
    fn from(list: NewList) -> Self {
        let NewList {
            destination,
            element_type,
            length_memory,
            length_index,
            element_size,
        } = list;

        InstructionBuilder::new(Operation::NEW_LIST)
            .operand_type(element_type)
            .a_field(destination)
            .b_memory(length_memory)
            .b_field(length_index)
            .c_field(element_size)
            .build()
    }
}

impl Display for NewList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let NewList {
            destination,
            element_type,
            length_memory,
            length_index,
            element_size: _,
        } = self;

        write!(
            f,
            "reg_{destination}: [{element_type}] = [{element_type}; {length_memory}{length_index}])"
        )
    }
}
