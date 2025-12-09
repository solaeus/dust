use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct SetList {
    pub destination_list: u16,
    pub item_source: Address,
    pub index: Address,
    pub item_type: OperandType,
}

impl From<&Instruction> for SetList {
    fn from(instruction: &Instruction) -> Self {
        let destination_list = instruction.a_field();
        let item_source = instruction.b_address();
        let index = instruction.c_address();
        let item_type = instruction.operand_type();

        SetList {
            destination_list,
            item_source,
            index,
            item_type,
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
        } = set_list.item_source;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = set_list.index;
        let operand_type = set_list.item_type;

        InstructionFields {
            operation,
            a_field,
            b_field,
            b_memory_kind,
            c_field,
            c_memory_kind,
            operand_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for SetList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let SetList {
            destination_list,
            item_source,
            index,
            item_type,
        } = self;

        write!(f, "reg_{destination_list}[")?;
        index.display(f, OperandType::INTEGER)?;
        write!(f, "] = ")?;
        item_source.display(f, *item_type)
    }
}
