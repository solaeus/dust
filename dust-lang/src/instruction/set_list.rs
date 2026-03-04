use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, Operation};

pub struct SetList {
    pub destination_list: u16,
    pub index: Address,
    pub source_operand: Address,
}

impl From<&Instruction> for SetList {
    fn from(instruction: &Instruction) -> Self {
        let destination_list = instruction.a_field();
        let source_operand = instruction.b_address();
        let index = instruction.c_address();

        SetList {
            destination_list,
            source_operand,
            index,
        }
    }
}

impl From<SetList> for Instruction {
    fn from(set_list: SetList) -> Self {
        let operation = Operation::SET_LIST;
        let a_field = set_list.destination_list;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = set_list.source_operand;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = set_list.index;

        InstructionFields {
            operation,
            a_field,
            b_field,
            b_memory_kind,
            c_field,
            c_memory_kind,
            ..Default::default()
        }
        .build()
    }
}

impl Display for SetList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let SetList {
            destination_list,
            source_operand,
            index,
        } = self;

        write!(f, "reg_{destination_list}[{index}] = {source_operand}")
    }
}
