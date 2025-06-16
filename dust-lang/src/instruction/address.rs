use std::fmt::{Formatter, FormattingOptions};

use serde::{Deserialize, Serialize};

use crate::r#type::TypeKind;

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

    pub fn display(&self, f: &mut Formatter, type_kind: TypeKind) -> std::fmt::Result {
        let index = self.index;

        match (type_kind, self.memory) {
            (TypeKind::Boolean, MemoryKind::CELL) => write!(f, "BOOL_Ce_{index}"),
            (TypeKind::Byte, MemoryKind::CELL) => write!(f, "BYTE_Ce_{index}"),
            (TypeKind::Character, MemoryKind::CELL) => write!(f, "CHAR_Ce_{index}"),
            (TypeKind::Float, MemoryKind::CELL) => write!(f, "FLOAT_Ce_{index}"),
            (TypeKind::Integer, MemoryKind::CELL) => write!(f, "INT_Ce_{index}"),
            (TypeKind::String, MemoryKind::CELL) => write!(f, "STR_Ce_{index}"),
            (TypeKind::List, MemoryKind::CELL) => write!(f, "LIST_Ce_{index}"),
            (TypeKind::Function, MemoryKind::CELL) => write!(f, "FN_Ce_{index}"),
            (TypeKind::Boolean, MemoryKind::CONSTANT) => write!(f, "{}", index != 0),
            (TypeKind::Byte, MemoryKind::CONSTANT) => write!(f, "{index:0x}"),
            (TypeKind::Character, MemoryKind::CONSTANT) => write!(f, "CHAR_Co_{index}"),
            (TypeKind::Float, MemoryKind::CONSTANT) => write!(f, "FLOAT_Co_{index}"),
            (TypeKind::Integer, MemoryKind::CONSTANT) => write!(f, "INT_Co_{index}"),
            (TypeKind::String, MemoryKind::CONSTANT) => write!(f, "STR_Co_{index}"),
            (TypeKind::Function, MemoryKind::CONSTANT) => write!(f, "FN_P_{index}"),
            (TypeKind::Boolean, MemoryKind::HEAP) => write!(f, "BOOL_H_{index}"),
            (TypeKind::Byte, MemoryKind::HEAP) => write!(f, "BYTE_H_{index}"),
            (TypeKind::Character, MemoryKind::HEAP) => write!(f, "CHAR_H_{index}"),
            (TypeKind::Float, MemoryKind::HEAP) => write!(f, "FLOAT_H_{index}"),
            (TypeKind::Integer, MemoryKind::HEAP) => write!(f, "INT_H_{index}"),
            (TypeKind::String, MemoryKind::HEAP) => write!(f, "STR_H_{index}"),
            (TypeKind::List, MemoryKind::HEAP) => write!(f, "LIST_H_{index}"),
            (TypeKind::Function, MemoryKind::HEAP) => write!(f, "FN_H_{index}"),
            (TypeKind::Boolean, MemoryKind::STACK) => write!(f, "BOOL_S_{index}"),
            (TypeKind::Byte, MemoryKind::STACK) => write!(f, "BYTE_S_{index}"),
            (TypeKind::Character, MemoryKind::STACK) => write!(f, "CHAR_S_{index}"),
            (TypeKind::Float, MemoryKind::STACK) => write!(f, "FLOAT_S_{index}"),
            (TypeKind::Integer, MemoryKind::STACK) => write!(f, "INT_S_{index}"),
            (TypeKind::String, MemoryKind::STACK) => write!(f, "STR_S_{index}"),
            (TypeKind::List, MemoryKind::STACK) => write!(f, "LIST_S_{index}"),
            (TypeKind::Function, MemoryKind::STACK) => write!(f, "FN_S_{index}"),
            (TypeKind::FunctionSelf, _) => write!(f, "FN_SELF"),
            _ => write!(f, "INVALID_ADDRESS_{index}"),
        }
    }

    pub fn to_string(&self, type_kind: TypeKind) -> String {
        let mut buffer = String::new();

        self.display(
            &mut Formatter::new(&mut buffer, FormattingOptions::new()),
            type_kind,
        )
        .unwrap();

        buffer
    }
}
