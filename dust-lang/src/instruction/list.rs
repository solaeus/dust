use std::fmt::{self, Display, Formatter};

use crate::r#type::TypeKind;

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct List {
    pub destination: Address,
    pub start: Address,
    pub end: Address,
    pub item_type: OperandType,
}

impl From<&Instruction> for List {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.destination();
        let start = instruction.b_address();
        let end = instruction.c_address();
        let item_type = instruction.operand_type();

        List {
            destination,
            start,
            end,
            item_type,
        }
    }
}

impl From<List> for Instruction {
    fn from(list: List) -> Self {
        let operation = Operation::LIST;
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = list.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = list.start;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = list.end;
        let operand_type = list.item_type;

        InstructionFields {
            operation,
            a_field,
            a_memory_kind,
            b_field,
            b_memory_kind,
            c_field,
            c_memory_kind,
            operand_type,
        }
        .build()
    }
}

impl Display for List {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let List {
            destination,
            start,
            end,
            item_type,
        } = self;
        let type_kind = match *item_type {
            OperandType::BOOLEAN => TypeKind::Boolean,
            OperandType::BYTE => TypeKind::Byte,
            OperandType::CHARACTER => TypeKind::Character,
            OperandType::FLOAT => TypeKind::Float,
            OperandType::INTEGER => TypeKind::Integer,
            OperandType::STRING => TypeKind::String,
            OperandType::LIST => TypeKind::List,
            OperandType::FUNCTION => TypeKind::Function,
            OperandType::NONE => TypeKind::None,
            _ => return write!(f, "INVALID_LIST_INSTRUCTION"),
        };

        destination.display(f, TypeKind::List)?;
        write!(f, " = [")?;
        start.display(f, type_kind)?;
        write!(f, "..=")?;
        end.display(f, type_kind)?;
        write!(f, "]")
    }
}
