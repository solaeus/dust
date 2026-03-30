use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, MemoryKind, OperandType, Operation};

pub struct GetIndex {
    pub destination: u16,
    pub operand_type: OperandType,
    pub base_register: u16,
    pub index_memory: MemoryKind,
    pub index_index: u16,
}

impl From<&Instruction> for GetIndex {
    fn from(instruction: &Instruction) -> Self {
        GetIndex {
            destination: instruction.a_field(),
            operand_type: instruction.operand_type(),
            base_register: instruction.b_field(),
            index_memory: instruction.c_memory(),
            index_index: instruction.c_field(),
        }
    }
}

impl From<GetIndex> for Instruction {
    fn from(get_index: GetIndex) -> Self {
        let GetIndex {
            destination,
            operand_type,
            base_register,
            index_memory,
            index_index,
        } = get_index;

        InstructionBuilder::new(Operation::GET_INDEX)
            .a_field(destination)
            .operand_type(operand_type)
            .b_field(base_register)
            .c_memory(index_memory)
            .c_field(index_index)
            .build()
    }
}

impl Display for GetIndex {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let GetIndex {
            destination,
            operand_type,
            base_register,
            index_memory,
            index_index,
        } = *self;

        write!(
            f,
            "reg_{destination}: {operand_type} = reg_{base_register}[{index_memory}_{index_index}]"
        )
    }
}
