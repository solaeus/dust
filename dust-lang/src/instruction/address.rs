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

    pub fn is_heap(&self) -> bool {
        self.memory == MemoryKind::HEAP
    }

    pub fn display(&self, f: &mut Formatter, type_kind: OperandType) -> std::fmt::Result {
        let index = self.index;

        match (type_kind, self.memory) {
            (OperandType::BOOLEAN, MemoryKind::CELL) => write!(f, "BOOL_Ce_{index}"),
            (OperandType::BYTE, MemoryKind::CELL) => write!(f, "BYTE_Ce_{index}"),
            (OperandType::CHARACTER, MemoryKind::CELL) => write!(f, "CHAR_Ce_{index}"),
            (OperandType::FLOAT, MemoryKind::CELL) => write!(f, "FLOAT_Ce_{index}"),
            (OperandType::INTEGER, MemoryKind::CELL) => write!(f, "INT_Ce_{index}"),
            (OperandType::STRING, MemoryKind::CELL) => write!(f, "STR_Ce_{index}"),
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
            ) => write!(f, "LIST_Ce_{index}"),
            (OperandType::FUNCTION, MemoryKind::CELL) => write!(f, "FN_Ce_{index}"),
            (OperandType::BOOLEAN, MemoryKind::CONSTANT) => write!(f, "{}", index != 0),
            (OperandType::BYTE, MemoryKind::CONSTANT) => write!(f, "{index:0x}"),
            (OperandType::CHARACTER, MemoryKind::CONSTANT) => write!(f, "CHAR_Co_{index}"),
            (OperandType::FLOAT, MemoryKind::CONSTANT) => write!(f, "FLOAT_Co_{index}"),
            (OperandType::INTEGER, MemoryKind::CONSTANT) => write!(f, "INT_Co_{index}"),
            (OperandType::STRING, MemoryKind::CONSTANT) => write!(f, "STR_Co_{index}"),
            (OperandType::FUNCTION, MemoryKind::CONSTANT) => write!(f, "FN_P_{index}"),
            (OperandType::BOOLEAN, MemoryKind::HEAP) => write!(f, "BOOL_H_{index}"),
            (OperandType::BYTE, MemoryKind::HEAP) => write!(f, "BYTE_H_{index}"),
            (OperandType::CHARACTER, MemoryKind::HEAP) => write!(f, "CHAR_H_{index}"),
            (OperandType::FLOAT, MemoryKind::HEAP) => write!(f, "FLOAT_H_{index}"),
            (OperandType::INTEGER, MemoryKind::HEAP) => write!(f, "INT_H_{index}"),
            (OperandType::STRING, MemoryKind::HEAP) => write!(f, "STR_H_{index}"),
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
            ) => write!(f, "LIST_H_{index}"),
            (OperandType::FUNCTION, MemoryKind::HEAP) => write!(f, "FN_H_{index}"),
            (OperandType::BOOLEAN, MemoryKind::STACK) => write!(f, "BOOL_S_{index}"),
            (OperandType::BYTE, MemoryKind::STACK) => write!(f, "BYTE_S_{index}"),
            (OperandType::CHARACTER, MemoryKind::STACK) => write!(f, "CHAR_S_{index}"),
            (OperandType::FLOAT, MemoryKind::STACK) => write!(f, "FLOAT_S_{index}"),
            (OperandType::INTEGER, MemoryKind::STACK) => write!(f, "INT_S_{index}"),
            (OperandType::FUNCTION, MemoryKind::STACK) => write!(f, "FN_SELF"),
            _ => write!(f, "INVALID_ADDRESS_{index}"),
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
