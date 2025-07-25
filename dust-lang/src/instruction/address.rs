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

    pub fn display_with_type(&self, operand_type: crate::instruction::OperandType) -> String {
        use crate::instruction::MemoryKind;
        use crate::instruction::OperandType;

        if self.memory == MemoryKind::CONSTANT && self.index == u16::MAX as usize {
            return "self".to_string();
        }

        match (self.memory, operand_type) {
            (MemoryKind::CONSTANT, OperandType::FUNCTION) => format!("proto_{}", self.index),
            (MemoryKind::CONSTANT, _) => format!("const_{}", self.index),
            (MemoryKind::REGISTER, _) => format!("reg_{}", self.index),
            (MemoryKind::ENCODED, _) => format!("enc_{}", self.index),
            (MemoryKind::CELL, _) => format!("cell_{}", self.index),
            _ => format!("invalid_memory_kind_{}", self.index),
        }
    }
}
