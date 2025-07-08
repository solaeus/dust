use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::Operation;

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct MemoryKind(pub u8);

impl MemoryKind {
    pub const REGISTER: MemoryKind = MemoryKind(0);
    pub const CONSTANT: MemoryKind = MemoryKind(1);
    pub const ENCODED: MemoryKind = MemoryKind(2);
    pub const CELL: MemoryKind = MemoryKind(3);
}

impl MemoryKind {
    pub fn invalid_panic(&self, ip: usize, operation: Operation) -> ! {
        panic!(
            "Invalid memory kind ({self}) at IP {ip} for {operation} operation. This is a bug in the compiler.",
        );
    }
}

impl Display for MemoryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::REGISTER => write!(f, "reg"),
            Self::ENCODED => write!(f, "enc"),
            Self::CONSTANT => write!(f, "const"),
            Self::CELL => write!(f, "cell"),
            _ => write!(f, "invalid_memory_kind"),
        }
    }
}
