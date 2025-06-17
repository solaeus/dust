use std::fmt::{Formatter, FormattingOptions};

use serde::{Deserialize, Serialize};

use crate::OperandType;

use super::MemoryKind;

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Address {
    pub index: u16,
    pub memory: MemoryKind,
}

impl Address {
    pub fn new(index: u16, memory: MemoryKind) -> Self {
        Self { index, memory }
    }

    pub fn cell(index: u16) -> Self {
        Address {
            index,
            memory: MemoryKind::CELL,
        }
    }

    pub fn constant(index: u16) -> Self {
        Address {
            index,
            memory: MemoryKind::CONSTANT,
        }
    }

    pub fn heap(index: u16) -> Self {
        Address {
            index,
            memory: MemoryKind::HEAP,
        }
    }

    pub fn function_self() -> Self {
        Address {
            index: u16::MAX,
            memory: MemoryKind::STACK,
        }
    }

    pub fn stack(index: u16) -> Self {
        Address {
            index,
            memory: MemoryKind::STACK,
        }
    }

    pub fn display(&self, f: &mut Formatter, type_kind: OperandType) -> std::fmt::Result {
        let index = self.index;

        match (type_kind, self.memory) {
            (OperandType::BOOLEAN, MemoryKind::CELL) => write!(f, "bool_ce_{index}"),
            (OperandType::BYTE, MemoryKind::CELL) => write!(f, "byte_ce_{index}"),
            (OperandType::CHARACTER, MemoryKind::CELL) => write!(f, "char_ce_{index}"),
            (OperandType::FLOAT, MemoryKind::CELL) => write!(f, "float_ce_{index}"),
            (OperandType::INTEGER, MemoryKind::CELL) => write!(f, "int_ce_{index}"),
            (OperandType::STRING, MemoryKind::CELL) => write!(f, "str_ce_{index}"),
            (
                OperandType::LIST_BOOLEAN
                | OperandType::LIST_BYTE
                | OperandType::LIST_CHARACTER
                | OperandType::LIST_FLOAT
                | OperandType::LIST_INTEGER
                | OperandType::LIST_STRING
                | OperandType::LIST_LIST
                | OperandType::LIST_FUNCTION,
                MemoryKind::CELL,
            ) => write!(f, "list_ce_{index}"),
            (OperandType::FUNCTION, MemoryKind::CELL) => write!(f, "fn_ce_{index}"),
            (OperandType::BOOLEAN, MemoryKind::CONSTANT) => write!(f, "{}", index != 0),
            (OperandType::BYTE, MemoryKind::CONSTANT) => write!(f, "{index:#04X}"),
            (OperandType::CHARACTER, MemoryKind::CONSTANT) => write!(f, "char_co_{index}"),
            (OperandType::FLOAT, MemoryKind::CONSTANT) => write!(f, "float_co_{index}"),
            (OperandType::INTEGER, MemoryKind::CONSTANT) => write!(f, "int_co_{index}"),
            (OperandType::STRING, MemoryKind::CONSTANT) => write!(f, "str_co_{index}"),
            (OperandType::FUNCTION, MemoryKind::CONSTANT) => write!(f, "fn_p_{index}"),
            (OperandType::BOOLEAN, MemoryKind::HEAP) => write!(f, "bool_h_{index}"),
            (OperandType::BYTE, MemoryKind::HEAP) => write!(f, "byte_h_{index}"),
            (OperandType::CHARACTER, MemoryKind::HEAP) => write!(f, "char_h_{index}"),
            (OperandType::FLOAT, MemoryKind::HEAP) => write!(f, "float_h_{index}"),
            (OperandType::INTEGER, MemoryKind::HEAP) => write!(f, "int_h_{index}"),
            (OperandType::STRING, MemoryKind::HEAP) => write!(f, "str_h_{index}"),
            (
                OperandType::LIST_BOOLEAN
                | OperandType::LIST_BYTE
                | OperandType::LIST_CHARACTER
                | OperandType::LIST_FLOAT
                | OperandType::LIST_INTEGER
                | OperandType::LIST_STRING
                | OperandType::LIST_LIST
                | OperandType::LIST_FUNCTION,
                MemoryKind::HEAP,
            ) => write!(f, "list_h_{index}"),
            (OperandType::FUNCTION, MemoryKind::HEAP) => write!(f, "fn_h_{index}"),
            (OperandType::BOOLEAN, MemoryKind::STACK) => write!(f, "bool_s_{index}"),
            (OperandType::BYTE, MemoryKind::STACK) => write!(f, "byte_s_{index}"),
            (OperandType::CHARACTER, MemoryKind::STACK) => write!(f, "char_s_{index}"),
            (OperandType::FLOAT, MemoryKind::STACK) => write!(f, "float_s_{index}"),
            (OperandType::INTEGER, MemoryKind::STACK) => write!(f, "int_s_{index}"),
            (OperandType::FUNCTION, MemoryKind::STACK) => write!(f, "fn_self"),
            _ => write!(f, "invalid_address_{index}"),
        }
    }

    pub fn to_string(&self, type_kind: OperandType) -> String {
        let mut buffer = String::new();

        self.display(
            &mut Formatter::new(&mut buffer, FormattingOptions::new()),
            type_kind,
        )
        .unwrap();

        buffer
    }
}
