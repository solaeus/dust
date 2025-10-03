use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct SetList {
    pub destination_list: Address,
    pub item_source: Address,
    pub list_index: u16,
    pub item_type: OperandType,
}

impl From<Instruction> for SetList {
    fn from(instruction: Instruction) -> Self {
        let destination_list = instruction.destination();
        let item_source = instruction.b_address();
        let list_index = instruction.c_field();
        let item_type = instruction.operand_type();

        SetList {
            destination_list,
            item_source,
            list_index,
            item_type,
        }
    }
}

impl From<SetList> for Instruction {
    fn from(set_list: SetList) -> Self {
        let operation = Operation::SET_LIST;
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = set_list.destination_list;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = set_list.item_source;
        let c_field = set_list.list_index;
        let operand_type = set_list.item_type;

        InstructionFields {
            operation,
            a_field,
            a_memory_kind,
            b_field,
            b_memory_kind,
            c_field,
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
            list_index,
            item_type,
        } = self;

        destination_list.display(f, item_type.list_type())?;
        write!(f, "[{list_index}] = ")?;
        item_source.display(f, *item_type)
    }
}
