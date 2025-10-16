use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct NewList {
    pub destination: Address,
    pub initial_length: u16,
    pub list_type: OperandType,
}

impl From<Instruction> for NewList {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let initial_length = instruction.b_field();
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
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = list.destination;
        let b_field = list.initial_length;
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
            list_type,
            ..
        } = self;
        let item_type = match *list_type {
            OperandType::LIST_BOOLEAN => OperandType::BOOLEAN.to_string(),
            OperandType::LIST_BYTE => OperandType::BYTE.to_string(),
            OperandType::LIST_CHARACTER => OperandType::CHARACTER.to_string(),
            OperandType::LIST_FLOAT => OperandType::FLOAT.to_string(),
            OperandType::LIST_INTEGER => OperandType::INTEGER.to_string(),
            OperandType::LIST_STRING => OperandType::STRING.to_string(),
            OperandType::LIST_LIST => "[[]]".to_string(),
            OperandType::LIST_FUNCTION => OperandType::FUNCTION.to_string(),
            _ => "error".to_string(),
        };

        destination.display(f, *list_type)?;
        write!(f, " = [{item_type}]")
    }
}
