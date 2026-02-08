use crate::{
    source::{Position, SourceFileId, Span},
    syntax::{
        SyntaxError, SyntaxId, SyntaxKind, SyntaxNode, SyntaxPayload, SyntaxTree,
        error::InternalSyntaxError,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct SyntaxReader<'a> {
    pub id: SyntaxId,
    tree: &'a SyntaxTree,
    node: &'a SyntaxNode,
}

impl<'a> SyntaxReader<'a> {
    pub fn new(id: SyntaxId, node: &'a SyntaxNode, tree: &'a SyntaxTree) -> Self {
        Self { id, node, tree }
    }

    pub fn root(&self) -> Result<Self, SyntaxError> {
        self.tree
            .root()
            .ok_or(SyntaxError::Internal(InternalSyntaxError::EmptySyntaxTree))
    }

    pub fn inner(&self) -> &'a SyntaxNode {
        self.node
    }

    pub fn kind(&self) -> SyntaxKind {
        self.node.kind
    }

    pub fn payload(&self) -> SyntaxPayload {
        self.node.payload
    }

    pub fn span(&self) -> Span {
        self.node.span
    }

    pub fn file_id(&self) -> SourceFileId {
        self.tree.file_id
    }

    pub fn position(&self) -> Position {
        Position::new(self.tree.file_id, self.node.span)
    }

    pub fn has_left_child(&self) -> bool {
        self.node.payload.left != SyntaxId::NONE.0
    }

    pub fn has_right_child(&self) -> bool {
        self.node.payload.right != SyntaxId::NONE.0
    }

    pub fn left_child(&self) -> Result<Self, SyntaxError> {
        let child_id = self.node.payload.left_id();
        let child_node = self.tree.get_node(child_id).ok_or(SyntaxError::Internal(
            InternalSyntaxError::MissingSyntaxNode(child_id),
        ))?;

        Ok(SyntaxReader::new(child_id, child_node, self.tree))
    }

    pub fn right_child(&self) -> Result<Self, SyntaxError> {
        let child_id = self.node.payload.right_id();
        let child_node = self.tree.get_node(child_id).ok_or(SyntaxError::Internal(
            InternalSyntaxError::MissingSyntaxNode(child_id),
        ))?;

        Ok(SyntaxReader::new(child_id, child_node, self.tree))
    }

    pub fn binary_children(&self) -> Result<(Self, Self), SyntaxError> {
        let left_child = self.left_child()?;
        let right_child = self.right_child()?;

        Ok((left_child, right_child))
    }

    pub fn multiple_children(&self) -> Result<SyntaxReaderIterator<'a>, SyntaxError> {
        let child_ids = self
            .tree
            .get_children(self.node.payload)
            .ok_or(SyntaxError::Internal(
                InternalSyntaxError::MissingSyntaxChildren(self.node.payload),
            ))?;

        Ok(SyntaxReaderIterator {
            child_ids,
            tree: self.tree,
            current_index: 0,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SyntaxReaderIterator<'a> {
    child_ids: &'a [SyntaxId],
    tree: &'a SyntaxTree,
    current_index: usize,
}

impl<'a> SyntaxReaderIterator<'a> {
    pub fn len(&self) -> usize {
        self.child_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.child_ids.is_empty()
    }

    pub fn expect_next(&mut self) -> Result<SyntaxReader<'a>, SyntaxError> {
        let child_id = *self
            .child_ids
            .get(self.current_index)
            .ok_or(SyntaxError::Internal(InternalSyntaxError::ExpectedChild))?;
        let child_node = self.tree.get_node(child_id).ok_or(SyntaxError::Internal(
            InternalSyntaxError::MissingSyntaxNode(child_id),
        ))?;
        self.current_index += 1;

        Ok(SyntaxReader::new(child_id, child_node, self.tree))
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

impl ExactSizeIterator for SyntaxReaderIterator<'_> {}
