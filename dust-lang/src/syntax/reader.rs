use tracing::info;

use crate::{
    source::Span,
    syntax::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxTree},
};

#[derive(Debug, Clone, Copy)]
pub struct SyntaxReader<'a> {
    pub id: SyntaxId,
    node: &'a SyntaxNode,
    tree: &'a SyntaxTree,
}

impl<'a> SyntaxReader<'a> {
    pub fn new(id: SyntaxId, node: &'a SyntaxNode, tree: &'a SyntaxTree) -> Self {
        Self { id, node, tree }
    }

    pub fn root(&self) -> Option<Self> {
        Some(SyntaxReader::new(
            SyntaxId(0),
            self.tree.nodes.first()?,
            self.tree,
        ))
    }

    pub fn inner(&self) -> &'a SyntaxNode {
        self.node
    }

    pub fn kind(&self) -> SyntaxKind {
        self.node.kind
    }

    pub fn span(&self) -> Span {
        self.node.span
    }

    pub fn has_left_child(&self) -> bool {
        self.node.children.0 != SyntaxId::NONE.0
    }

    pub fn has_right_child(&self) -> bool {
        self.node.children.1 != SyntaxId::NONE.0
    }

    pub fn left_child(&self) -> Option<Self> {
        let child_id = SyntaxId(self.node.children.0);
        let child_node = self.tree.get_node(child_id)?;

        Some(SyntaxReader::new(child_id, child_node, self.tree))
    }

    pub fn right_child(&self) -> Option<Self> {
        let child_id = SyntaxId(self.node.children.1);
        let child_node = self.tree.get_node(child_id)?;

        Some(SyntaxReader::new(child_id, child_node, self.tree))
    }

    pub fn binary_children(&self) -> Option<(Self, Self)> {
        let left_child = self.left_child()?;
        let right_child = self.right_child()?;

        Some((left_child, right_child))
    }

    pub fn multiple_children(&self) -> Option<SyntaxReaderIterator<'a>> {
        let child_ids = self
            .tree
            .get_children(self.node.children.0, self.node.children.1)?;

        Some(SyntaxReaderIterator {
            child_ids,
            tree: self.tree,
            current_index: 0,
        })
    }
}

pub struct SyntaxReaderIterator<'a> {
    child_ids: &'a [SyntaxId],
    tree: &'a SyntaxTree,
    current_index: usize,
}

impl<'a> SyntaxReaderIterator<'a> {
    pub fn len(&self) -> usize {
        self.child_ids.len()
    }
}

impl<'a> Iterator for SyntaxReaderIterator<'a> {
    type Item = SyntaxReader<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let child_id = *self.child_ids.get(self.current_index)?;
        let child_node = self.tree.get_node(child_id)?;
        self.current_index += 1;

        Some(SyntaxReader::new(child_id, child_node, self.tree))
    }
}
