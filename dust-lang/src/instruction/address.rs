use std::fmt::{self, Formatter};

use serde::{Deserialize, Serialize};

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

    pub fn encoded(index: usize) -> Self {
        Address {
            index,
            memory: MemoryKind::ENCODED,
        }
    }

    pub fn function_self() -> Self {
        Address {
            index: usize::MAX,
            memory: MemoryKind::CONSTANT,
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.memory == MemoryKind::CONSTANT && self.index == u16::MAX as usize {
            write!(f, "self")
        } else {
            write!(f, "{}_{}", self.memory, self.index)
        }
    }
}
