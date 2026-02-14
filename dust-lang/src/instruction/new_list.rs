use std::fmt::{self, Display, Formatter};

use crate::instruction::Address;

use super::{ByteType, Instruction, InstructionFields, Operation};

pub struct NewList {
    pub destination: u16,
    pub initial_length: Address,
    pub list_type: ByteType,
}

impl From<&Instruction> for NewList {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let initial_length = instruction.b_address();
        let list_type = instruction.operand_type();

        NewList {
            destination,
            initial_length,
            list_type,
        }
    }
}

impl From<NewList> for Instruction {
    fn from(list: NewList) -> Self {
        let operation = Operation::NEW_LIST;
        let a_field = list.destination;
        let b_field = list.initial_length.index;
        let b_memory_kind = list.initial_length.memory;
        let operand_type = list.list_type;

        InstructionFields {
            operation,
            a_field,
            b_field,
            b_memory_kind,
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
            initial_length,
            list_type,
        } = self;
        let item_type = match *list_type {
            ByteType::LIST_BOOLEAN => &ByteType::BOOLEAN.to_string(),
            ByteType::LIST_BYTE => &ByteType::BYTE.to_string(),
            ByteType::LIST_CHARACTER => &ByteType::CHARACTER.to_string(),
            ByteType::LIST_FLOAT => &ByteType::FLOAT.to_string(),
            ByteType::LIST_INTEGER => &ByteType::INTEGER.to_string(),
            ByteType::LIST_STRING => &ByteType::STRING.to_string(),
            ByteType::LIST_LIST => "[[]]",
            ByteType::LIST_FUNCTION => &ByteType::FUNCTION.to_string(),
            _ => "error",
        };

        write!(f, "reg_{destination} = [{item_type}; ")?;
        initial_length.display(f, *list_type)?;
        write!(f, "]")
    }
}
