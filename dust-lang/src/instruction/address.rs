use std::fmt::{self, Formatter, FormattingOptions};

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

    pub fn register(index: u16) -> Self {
        Address {
            index,
            memory: MemoryKind::REGISTER,
        }
    }

    pub fn encoded(index: u16) -> Self {
        Address {
            index,
            memory: MemoryKind::ENCODED,
        }
    }

    pub fn function_self() -> Self {
        Address {
            index: u16::MAX,
            memory: MemoryKind::CONSTANT,
        }
    }

    pub fn display(&self, f: &mut Formatter<'_>, r#type: OperandType) -> fmt::Result {
        if r#type == OperandType::FUNCTION {
            if self.index == u16::MAX {
                write!(f, "self")
            } else {
                write!(f, "proto_{}", self.index)
            }
        } else {
            write!(f, "{}_{}", self.memory, self.index)
        }
    }

    pub fn to_string(&self, r#type: OperandType) -> String {
        let mut string = String::new();
        let mut formatter = Formatter::new(&mut string, FormattingOptions::default());

        self.display(&mut formatter, r#type).unwrap();

        string
    }
}
