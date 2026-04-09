pub mod components;
pub mod error;
pub mod node;
pub mod reader;
pub mod tree;
pub mod visitor;

use serde::{Deserialize, Serialize};

use crate::source::FileId;

use error::SyntaxError;
use tree::SyntaxTree;

#[derive(Debug)]
pub struct Syntax {
    trees: Vec<Option<SyntaxTree>>,
}

impl Syntax {
    pub fn new(length: usize) -> Self {
        Self {
            trees: vec![None; length],
        }
    }

    pub fn len(&self) -> usize {
        self.trees.len()
    }

    pub fn is_empty(&self) -> bool {
        self.trees.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &SyntaxTree> {
        self.trees.iter().filter_map(|tree| tree.as_ref())
    }

    pub fn add_tree(&mut self, tree: SyntaxTree) {
        let index = tree.file_id.inner() as usize;

        if index < self.trees.len() {
            self.trees[index] = Some(tree);
        }
    }

    pub fn get_tree(&self, file_id: FileId) -> Result<&SyntaxTree, SyntaxError> {
        let index = file_id.inner() as usize;

        self.trees
            .get(index)
            .and_then(|tree| tree.as_ref())
            .ok_or(SyntaxError::MissingSyntaxTree(file_id))
    }
}

/// A unique identifier for a syntax node within a syntax tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SyntaxId(#[cfg(test)] pub(super) u32, #[cfg(not(test))] u32);

impl SyntaxId {
    /// ID of the root node of a syntax tree, which is always 0 because nodes are added in lexical
    /// order.
    pub const ROOT: SyntaxId = SyntaxId(0);

    /// ID representing the absence of a syntax node.
    pub const NONE: SyntaxId = SyntaxId(u32::MAX);

    pub fn inner(&self) -> u32 {
        self.0
    }
}
