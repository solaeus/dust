use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{Address, Type};

/// Block-scoped variable.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Local {
    /// Where the variable's value is stored.
    pub address: Address,

    /// Type of the variable's value.
    pub r#type: Type,

    /// Whether the local is mutable.
    pub is_mutable: bool,

    /// Scope where the variable was declared.
    pub scope: BlockScope,
}

impl Local {
    /// Creates a new Local instance.
    pub fn new(address: Address, r#type: Type, is_mutable: bool, scope: BlockScope) -> Self {
        Self {
            address,
            r#type,
            is_mutable,
            scope,
        }
    }
}

/// Variable locality, as defined by its depth and block index within a function.
///
/// The `block index` is a unique identifier for a block within a chunk. It is used to differentiate
/// between blocks that are not nested together but have the same depth, i.e. sibling scopes. If the
/// `block_index` is 0, then the scope is the root scope of the chunk. The `block_index` is always 0
/// when the `depth` is 0.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BlockScope {
    /// Level of block nesting.
    pub block_depth: u8,
    /// Index of the block in the chunk.
    pub block_index: u8,
}

impl BlockScope {
    pub fn new(block_depth: u8, block_index: u8) -> Self {
        Self {
            block_depth,
            block_index,
        }
    }

    pub fn contains(&self, other: &Self) -> bool {
        match self.block_depth.cmp(&other.block_depth) {
            Ordering::Less => false,
            Ordering::Greater => self.block_index >= other.block_index,
            Ordering::Equal => self.block_index == other.block_index,
        }
    }

    pub fn begin(&mut self, block_index: u8) {
        self.block_depth += 1;
        self.block_index = block_index;
    }

    pub fn end(&mut self, block_index: u8) {
        self.block_depth -= 1;
        self.block_index = block_index;
    }
}

impl Display for BlockScope {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.block_depth, self.block_index)
    }
}
