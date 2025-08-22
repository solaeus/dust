use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

/// Variable and expression locality, as defined by the nesting depth and block index within a
/// function.
///
/// The `block index` is a unique identifier for a block within a chunk. It is used to differentiate
/// between blocks that are not nested together but have the same depth, i.e. sibling scopes. If the
/// `block_index` is 0, then the scope is the root scope of the chunk.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Scope {
    /// Level of block nesting.
    pub depth: u8,

    /// Index of the block in the chunk.
    pub block_index: u8,
}

impl Scope {
    pub fn contains(&self, other: &Self) -> bool {
        match self.depth.cmp(&other.depth) {
            Ordering::Less => false,
            Ordering::Greater => self.block_index >= other.block_index,
            Ordering::Equal => self.block_index == other.block_index,
        }
    }

    pub fn begin(&mut self, block_index: u8) {
        self.depth += 1;
        self.block_index = block_index;
    }

    pub fn end(&mut self, block_index: u8) {
        self.depth -= 1;
        self.block_index = block_index;
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.depth, self.block_index)
    }
}
