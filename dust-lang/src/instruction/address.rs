use std::fmt::{self, Formatter, FormattingOptions};

use serde::{Deserialize, Serialize};

use crate::OperandType;

use super::MemoryKind;

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Address {
    pub index: usize,
    pub memory: MemoryKind,
}

impl Address {
    pub fn new(index: usize, memory: MemoryKind) -> Self {
        Self { index, memory }
    }

    pub fn cell(index: usize) -> Self {
        Address {
            index,
            memory: MemoryKind::CELL,
        }
    }

    pub fn constant(index: usize) -> Self {
        Address {
            index,
            memory: MemoryKind::CONSTANT,
        }
    }

    pub fn register(index: usize) -> Self {
        Address {
            index,
            memory: MemoryKind::REGISTER,
        }
    }

    pub fn function_self() -> Self {
        Address {
            index: usize::MAX,
            memory: MemoryKind::CONSTANT,
        }
    }

    pub fn display(&self, f: &mut Formatter, r#type: OperandType) -> fmt::Result {
        write!(
            f,
            "{type}_{memory}_{index}",
            memory = self.memory,
            index = self.index
        )
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
