mod error;
mod node;
mod reader;
mod tree;
mod visitor;

pub use error::SyntaxError;
pub use node::{SyntaxKind, SyntaxNode, SyntaxNodeChildren, SyntaxPayload};
pub use reader::{SyntaxReader, SyntaxReaderIterator};
pub use tree::SyntaxTree;
pub use visitor::SyntaxVisitor;

use crate::source::SourceFileId;

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

    pub fn add_tree(&mut self, tree: SyntaxTree) -> Result<(), usize> {
        let index = tree.file_id.inner() as usize;

        if index < self.trees.len() {
            self.trees[index] = Some(tree);

            Ok(())
        } else {
            Err(self.trees.len())
        }
    }

    pub fn get_tree(&self, file_id: SourceFileId) -> Option<&SyntaxTree> {
        let index = file_id.inner() as usize;

        if index < self.trees.len() {
            self.trees[index].as_ref()
        } else {
            None
        }
    }
}

/// A unique identifier for a syntax node within a syntax tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxId(pub(super) u32);

impl SyntaxId {
    /// ID of the root node of a syntax tree, which is always 0 because nodes are added in lexical
    /// order.
    pub const ROOT: SyntaxId = SyntaxId(0);

    /// ID representing the absence of a syntax node.
    pub const NONE: SyntaxId = SyntaxId(u32::MAX);

    pub fn inner(&self) -> u32 {
        self.0
    }

    pub fn is_none(&self) -> bool {
        *self == SyntaxId::NONE
    }
}
