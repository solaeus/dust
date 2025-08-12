use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct NewList {
    pub destination: Address,
    pub length: u16,
    pub list_type: OperandType,
}

impl From<Instruction> for NewList {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let length = instruction.b_field();
        let list_type = instruction.operand_type();

        NewList {
            destination,
            length,
            list_type,
        }
    }
}

impl From<NewList> for Instruction {
    fn from(list: NewList) -> Self {
        let operation = Operation::NEW_LIST;
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = list.destination;
        let b_field = list.length;
        let operand_type = list.list_type;

        InstructionFields {
            operation,
            a_field,
            a_memory_kind,
            b_field,
            operand_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for NewList {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let NewList {
            destination,
            length,
            list_type,
        } = self;
        let item_type = list_type
            .list_item_type()
            .map(|item_type| item_type.to_string())
            .unwrap_or_else(|| "error".to_string());

        destination.display(f, *list_type)?;
        write!(f, " = [{item_type}; {length}]")
    }
}
