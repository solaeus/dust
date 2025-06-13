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

    pub fn stack(index: u16) -> Self {
        Address {
            index,
            memory: MemoryKind::STACK,
        }
    }

    pub fn display(&self, f: &mut Formatter, type_kind: TypeKind) -> std::fmt::Result {
        let index = self.index;

        match (type_kind, self.memory) {
            (TypeKind::Boolean, MemoryKind::CELL) => write!(f, "BOOL_CELL_{index}"),
            (TypeKind::Boolean, MemoryKind::CONSTANT) => write!(f, "BOOL_CONST_{index}"),
            (TypeKind::Boolean, MemoryKind::HEAP) => write!(f, "BOOL_HEAP_{index}"),
            (TypeKind::Boolean, MemoryKind::STACK) => write!(f, "BOOL_STACK_{index}"),
            (TypeKind::Byte, MemoryKind::CELL) => write!(f, "BYTE_CELL_{index}"),
            (TypeKind::Byte, MemoryKind::CONSTANT) => write!(f, "BYTE_CONST_{index}"),
            (TypeKind::Byte, MemoryKind::HEAP) => write!(f, "BYTE_HEAP_{index}"),
            (TypeKind::Byte, MemoryKind::STACK) => write!(f, "BYTE_STACK_{index}"),
            (TypeKind::Character, MemoryKind::CELL) => write!(f, "CHAR_CELL_{index}"),
            (TypeKind::Character, MemoryKind::CONSTANT) => write!(f, "CHAR_CONST_{index}"),
            (TypeKind::Character, MemoryKind::HEAP) => write!(f, "CHAR_HEAP_{index}"),
            (TypeKind::Character, MemoryKind::STACK) => write!(f, "CHAR_STACK_{index}"),
            (TypeKind::Float, MemoryKind::CELL) => write!(f, "FLOAT_CELL_{index}"),
            (TypeKind::Float, MemoryKind::CONSTANT) => write!(f, "FLOAT_CONST_{index}"),
            (TypeKind::Float, MemoryKind::HEAP) => write!(f, "FLOAT_HEAP_{index}"),
            (TypeKind::Float, MemoryKind::STACK) => write!(f, "FLOAT_STACK_{index}"),
            (TypeKind::Integer, MemoryKind::CELL) => write!(f, "INT_CELL_{index}"),
            (TypeKind::Integer, MemoryKind::CONSTANT) => write!(f, "INT_CONST_{index}"),
            (TypeKind::Integer, MemoryKind::HEAP) => write!(f, "INT_HEAP_{index}"),
            (TypeKind::Integer, MemoryKind::STACK) => write!(f, "INT_STACK_{index}"),
            (TypeKind::String, MemoryKind::CELL) => write!(f, "STRING_CELL_{index}"),
            (TypeKind::String, MemoryKind::CONSTANT) => write!(f, "STRING_CONST_{index}"),
            (TypeKind::String, MemoryKind::HEAP) => write!(f, "STRING_HEAP_{index}"),
            (TypeKind::String, MemoryKind::STACK) => write!(f, "STRING_STACK_{index}"),
            (TypeKind::List, MemoryKind::CELL) => write!(f, "LIST_CELL_{index}"),
            (TypeKind::List, MemoryKind::CONSTANT) => write!(f, "LIST_CONST_{index}"),
            (TypeKind::List, MemoryKind::HEAP) => write!(f, "LIST_HEAP_{index}"),
            (TypeKind::List, MemoryKind::STACK) => write!(f, "LIST_STACK_{index}"),
            (TypeKind::Function, MemoryKind::CELL) => write!(f, "FN_CELL_{index}"),
            (TypeKind::Function, MemoryKind::CONSTANT) => write!(f, "FN_CONST_{index}"),
            (TypeKind::Function, MemoryKind::HEAP) => write!(f, "FN_HEAP_{index}"),
            (TypeKind::Function, MemoryKind::STACK) => write!(f, "FN_STACK_{index}"),
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
