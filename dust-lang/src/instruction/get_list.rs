use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct GetList {
    pub destination: u16,
    pub list: Address,
    pub list_index: Address,
    pub item_type: OperandType,
}

impl From<&Instruction> for GetList {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let item_source = instruction.b_address();
        let list_index = instruction.c_address();
        let item_type = instruction.operand_type();

        GetList {
            destination,
            list: item_source,
            list_index,
            item_type,
        }
    }
}

impl From<GetList> for Instruction {
    fn from(set_list: GetList) -> Self {
        let operation = Operation::GET_LIST;
        let a_field = set_list.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = set_list.list;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = set_list.list_index;
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

impl Display for GetList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let GetList {
            destination,
            list,
            list_index,
            item_type,
        } = self;

        write!(f, "reg_{destination} = ")?;
        list.display(f, item_type.list_type())?;
        write!(f, "[")?;
        list_index.display(f, *item_type)?;
        write!(f, "]")
    }
}
