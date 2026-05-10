pub mod components;
pub mod error;
pub mod node;
pub mod reader;
pub mod tree;

use serde::{Deserialize, Serialize};

use crate::source::SourceCodeId;

use error::SyntaxError;
use tree::SyntaxTree;

#[derive(Debug)]
pub struct Syntax {
    trees: Vec<SyntaxTree>,
}

impl Syntax {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            trees: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.trees.len()
    }

    pub fn is_empty(&self) -> bool {
        self.trees.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &SyntaxTree> {
        self.trees.iter()
    }

    pub fn add_tree(&mut self, tree: SyntaxTree) {
        let index = tree.source_id.inner() as usize;

        debug_assert!(
            index < self.trees.len() || self.trees.is_empty(),
            "SyntaxTrees must be added in order by SourceId"
        );

        self.trees.push(tree);
    }

    pub fn get_tree(&self, source_id: SourceCodeId) -> Result<&SyntaxTree, SyntaxError> {
        let index = source_id.inner() as usize;

        self.trees
            .get(index)
            .ok_or(SyntaxError::MissingTree(source_id))
    }
}

/// A unique identifier for a syntax node within a syntax tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SyntaxId(#[cfg(test)] pub(super) u32, #[cfg(not(test))] u32);

impl SyntaxId {
    /// ID of the root node of a syntax tree, which is always 0 because nodes are added in lexical
    /// order.
    pub const ROOT: SyntaxId = SyntaxId(0);

    pub fn inner(&self) -> u32 {
        self.0
    }
}
