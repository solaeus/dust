use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, Operation};

pub struct GetList {
    pub destination: u16,
    pub list: Address,
    pub list_index: Address,
}

impl From<&Instruction> for GetList {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let item_source = instruction.b_address();
        let list_index = instruction.c_address();

        GetList {
            destination,
            list: item_source,
            list_index,
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

impl Display for GetList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let GetList {
            destination,
            list,
            list_index,
        } = self;

        write!(f, "reg_{destination} = {list}[{list_index}]")
    }
}
