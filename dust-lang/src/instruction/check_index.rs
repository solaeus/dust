use std::fmt::{self, Display, Formatter};

use crate::instruction::{Instruction, InstructionBuilder, MemoryKind, Operation};

pub struct CheckIndex {
    pub index_memory: MemoryKind,
    pub index_index: u16,
    pub array_length: u16,
}

impl From<&Instruction> for CheckIndex {
    fn from(instruction: &Instruction) -> Self {
        CheckIndex {
            index_memory: instruction.b_memory(),
            index_index: instruction.b_field(),
            array_length: instruction.c_field(),
        }
    }
}

impl From<CheckIndex> for Instruction {
    fn from(check_index: CheckIndex) -> Self {
        let CheckIndex {
            index_memory,
            index_index,
            array_length,
        } = check_index;

        InstructionBuilder::new(Operation::CHECK_INDEX)
            .b_memory(index_memory)
            .b_field(index_index)
            .c_field(array_length)
            .build()
    }
}

impl Display for CheckIndex {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let CheckIndex {
            index_memory,
            index_index,
            array_length,
        } = *self;

        write!(f, "{index_memory}_{index_index} < {array_length}")
    }
}
