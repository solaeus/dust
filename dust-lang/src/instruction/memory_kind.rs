use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct MemoryKind(pub u8);

impl MemoryKind {
    pub const CELL: MemoryKind = MemoryKind(0);
    pub const CONSTANT: MemoryKind = MemoryKind(1);
    pub const HEAP: MemoryKind = MemoryKind(2);
    pub const STACK: MemoryKind = MemoryKind(3);
}
