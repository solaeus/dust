mod error;
mod node;
mod reader;
mod tree;
mod visitor;

pub use error::SyntaxError;
pub use node::{SyntaxKind, SyntaxNode, SyntaxNodeChildren, SyntaxPayload};
pub use reader::{SyntaxReader, SyntaxReaderIterator};
pub use tree::SyntaxTree;
pub use visitor::{
    ExpressionVisitor, ItemVisitor, OtherVisitor, StatementVisitor, SyntaxVisitor,
    SyntaxVistorTypes,
};

use crate::source::SourceFileId;

#[derive(Debug)]
pub struct Syntax {
    file_trees: Vec<SyntaxTree>,
}

impl Syntax {
    pub fn new() -> Self {
        Self {
            file_trees: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            file_trees: Vec::with_capacity(capacity),
        }
    }

    pub fn file_count(&self) -> usize {
        self.file_trees.len()
    }

    pub fn add_tree(&mut self, tree: SyntaxTree) -> SourceFileId {
        let id = SourceFileId(self.file_trees.len() as u32);

        self.file_trees.push(tree);

        id
    }

    pub fn get_tree(&self, file_id: SourceFileId) -> Option<&SyntaxTree> {
        self.file_trees.get(file_id.0 as usize)
    }
}

impl Default for Syntax {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxId(pub u32);

impl SyntaxId {
    pub const ROOT: SyntaxId = SyntaxId(0);
    pub const NONE: SyntaxId = SyntaxId(u32::MAX);

    pub fn is_none(&self) -> bool {
        *self == SyntaxId::NONE
    }
}
