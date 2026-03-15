use crate::{
    source::{Position, SourceFileId, Span},
    syntax::{
        SyntaxId,
        components::SyntaxComponent,
        error::SyntaxError,
        node::{SyntaxKind, SyntaxNode, SyntaxPayload, SyntaxPayloadKind},
        tree::SyntaxTree,
    },
};

#[derive(Clone, Copy, Debug)]
pub struct SyntaxReader<'a> {
    pub id: SyntaxId,
    tree: &'a SyntaxTree,
    pub node: &'a SyntaxNode,
}

impl<'a> SyntaxReader<'a> {
    pub fn new(id: SyntaxId, node: &'a SyntaxNode, tree: &'a SyntaxTree) -> Self {
        Self { id, node, tree }
    }

    pub fn root(&self) -> Result<Self, SyntaxError> {
        self.tree.root()
    }

    pub fn node(&self) -> &'a SyntaxNode {
        self.node
    }

    pub fn kind(&self) -> SyntaxKind {
        self.node.kind
    }

    pub fn children_payload(&self) -> SyntaxPayload {
        self.node.children
    }

    pub fn span(&self) -> Span {
        self.node.span
    }

    pub fn modifier(&self) -> bool {
        self.node.modifier
    }

    pub fn is_item(&self) -> bool {
        self.node.kind.is_item()
    }

    pub fn is_statement(&self) -> bool {
        self.node.kind.is_statement()
    }

    pub fn is_expression(&self) -> bool {
        self.node.kind.is_expression()
    }

    pub fn file_id(&self) -> SourceFileId {
        self.tree.file_id
    }

    pub fn position(&self) -> Position {
        Position::new(self.tree.file_id, self.node.span)
    }

    pub fn child_count(&self) -> usize {
        match self.node.children_kind {
            SyntaxPayloadKind::SingleChild => 1,
            SyntaxPayloadKind::BinaryChildren => 2,
            SyntaxPayloadKind::MultipleChildren => self.node.children.right as usize,
            _ => 0,
        }
    }

    pub fn has_left_child(&self) -> bool {
        matches!(
            self.node.children_kind,
            SyntaxPayloadKind::SingleChild
                | SyntaxPayloadKind::BinaryChildren
                | SyntaxPayloadKind::MultipleChildren
        )
    }

    pub fn has_right_child(&self) -> bool {
        matches!(
            self.node.children_kind,
            SyntaxPayloadKind::BinaryChildren | SyntaxPayloadKind::MultipleChildren
        )
    }

    pub fn single_child(&self) -> Result<Self, SyntaxError> {
        debug_assert!(self.node.children_kind == SyntaxPayloadKind::SingleChild);

        let left_id = self.node.children.left_id();
        let left = self.tree.get_node(left_id)?;

        Ok(left)
    }

    pub fn binary_children(&self) -> Result<(Self, Self), SyntaxError> {
        debug_assert!(self.node.children_kind == SyntaxPayloadKind::BinaryChildren);

        let left_id = self.node.children.left_id();
        let right_id = self.node.children.right_id();

        let left_child = self.tree.get_node(left_id)?;
        let right_child = self.tree.get_node(right_id)?;

        Ok((left_child, right_child))
    }

    pub fn single_or_binary_children(&self) -> Result<(Self, Option<Self>), SyntaxError> {
        debug_assert!(matches!(
            self.node.children_kind,
            SyntaxPayloadKind::SingleChild | SyntaxPayloadKind::BinaryChildren
        ));

        let left_id = self.node.children.left_id();
        let left_child = self.tree.get_node(left_id)?;

        let right_id = self.node.children.right_id();
        let right_child = if right_id == SyntaxId::NONE {
            None
        } else {
            Some(self.tree.get_node(right_id)?)
        };

        Ok((left_child, right_child))
    }

    pub fn children(&'a self) -> SyntaxReaderIterator<'a> {
        SyntaxReaderIterator::new(self)
    }

    pub fn last_child(&'a self) -> Result<Option<Self>, SyntaxError> {
        match self.node.children_kind {
            SyntaxPayloadKind::SingleChild => self.single_child().map(Some),
            SyntaxPayloadKind::BinaryChildren => {
                let right_id = self.node.children.right_id();

                self.tree.get_node(right_id).map(Some)
            }
            SyntaxPayloadKind::MultipleChildren => Ok(SyntaxReaderIterator::new(self).next_back()),
            _ => Ok(None),
        }
    }

    pub fn as_component<T: SyntaxComponent<'a>>(&'a self) -> Result<T, SyntaxError> {
        T::from_reader(self)
    }

    pub fn draw_text_tree(&self, buffer: &mut String) {
        let mut ancestors = Vec::new();

        buffer.push_str(self.node.kind.as_str());
        buffer.push('\n');

        let size = self.child_count();

        if size > 0 {
            for (index, child) in self.children().enumerate() {
                let child_is_last = index == size.saturating_sub(1);

                child.draw_text_tree_line(buffer, &mut ancestors, child_is_last);
            }
        }
    }

    fn draw_text_tree_line(&self, buffer: &mut String, ancestors: &mut Vec<bool>, is_last: bool) {
        for ancestor_is_last in &*ancestors {
            let indent = if *ancestor_is_last { "    " } else { "│   " };

            buffer.push_str(indent);
        }

        let connector = if is_last { "└── " } else { "├── " };

        buffer.push_str(connector);
        buffer.push_str(self.node.kind.as_str());
        buffer.push('\n');

        let size = self.child_count();

        ancestors.push(is_last);

        if size == 0 {
            ancestors.pop();

            return;
        }

        for (index, child) in self.children().enumerate() {
            let child_is_last = index == size.saturating_sub(1);

            child.draw_text_tree_line(buffer, ancestors, child_is_last);
        }

        ancestors.pop();
    }
}

#[derive(Debug)]
pub struct SyntaxReaderIterator<'a> {
    parent: &'a SyntaxReader<'a>,
    current_index: usize,
}

impl<'a> SyntaxReaderIterator<'a> {
    fn new(parent: &'a SyntaxReader<'a>) -> Self {
        Self {
            parent,
            current_index: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self.parent.node.children_kind {
            SyntaxPayloadKind::Empty => true,
            SyntaxPayloadKind::SingleChild
            | SyntaxPayloadKind::BinaryChildren
            | SyntaxPayloadKind::MultipleChildren => false,
        }
    }

    pub fn expect_next(&mut self) -> Result<SyntaxReader<'a>, SyntaxError> {
        self.next().ok_or_else(|| SyntaxError::MissingSyntaxChild {
            total_children: self.parent.child_count(),
        })
    }
}

impl<'a> Iterator for SyntaxReaderIterator<'a> {
    type Item = SyntaxReader<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = match self.parent.node.children_kind {
            SyntaxPayloadKind::SingleChild if self.current_index == 0 => {
                let child_id = self.parent.node.children.left_id();
                self.current_index += 1;

                child_id
            }
            SyntaxPayloadKind::BinaryChildren => {
                let child_id = if self.current_index == 0 {
                    self.parent.node.children.left_id()
                } else if self.current_index == 1 {
                    self.parent.node.children.right_id()
                } else {
                    return None;
                };
                self.current_index += 1;

                child_id
            }
            SyntaxPayloadKind::MultipleChildren => {
                let child_index = self.parent.children_payload().left as usize + self.current_index;

                if child_index >= self.parent.children_payload().right as usize {
                    return None;
                }

                self.current_index += 1;

                self.parent.tree.children[child_index]
            }
            _ => return None,
        };
        let node = &self.parent.tree.nodes[id.0 as usize];

        Some(SyntaxReader::new(id, node, self.parent.tree))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let start_length = self.parent.child_count();
        let remaining = start_length.saturating_sub(self.current_index);

        (remaining, Some(remaining))
    }
}

impl DoubleEndedIterator for SyntaxReaderIterator<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let child_id = match self.parent.node.children_kind {
            SyntaxPayloadKind::SingleChild if self.current_index == 0 => {
                let child_id = self.parent.node.children.left_id();
                self.current_index += 1;

                child_id
            }
            SyntaxPayloadKind::BinaryChildren if self.current_index < 2 => {
                let child_id = if self.current_index == 0 {
                    self.parent.node.children.left_id()
                } else {
                    self.parent.node.children.right_id()
                };
                self.current_index += 1;

                child_id
            }
            SyntaxPayloadKind::MultipleChildren
                if self.current_index < self.parent.child_count() =>
            {
                let child_index =
                    self.parent.children_payload().right as usize - self.current_index - 1;
                self.current_index += 1;

                self.parent.tree.children[child_index]
            }
            _ => return None,
        };

        self.parent.tree.get_node(child_id).ok()
    }
}

impl ExactSizeIterator for SyntaxReaderIterator<'_> {}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use crate::parser::parse;

    #[test]
    fn double_ended_iterator() {
        let (syntax_tree, errors) = parse(
            "fn main() -> i32 { 1 + 2 * 3 } fn foo() -> i32 { 4 - 5 / 6 } fn bar() -> i32 { 7 % 8 }",
        );

        assert!(errors.is_empty());

        let root = syntax_tree.root().unwrap();

        let forward = root
            .children()
            .map(|reader| reader.node())
            .collect::<Vec<_>>();
        let backward = root
            .children()
            .rev()
            .map(|reader| reader.node())
            .collect::<Vec<_>>();

        for (forward_node, backward_node) in forward.iter().zip(backward.iter().rev()) {
            assert_eq!(forward_node, backward_node);
        }
    }
}
