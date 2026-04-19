use crate::{
    source::{FileId, Position},
    syntax::{
        SyntaxId,
        components::SyntaxComponent,
        error::SyntaxError,
        node::{SyntaxChildrenKind, SyntaxKind, SyntaxNode},
        tree::SyntaxTree,
    },
};

#[derive(Clone, Copy, Debug)]
pub struct SyntaxReader<'a> {
    pub id: SyntaxId,
    pub node: SyntaxNode,
    tree: &'a SyntaxTree,
}

impl<'a> SyntaxReader<'a> {
    pub fn new(id: SyntaxId, node: SyntaxNode, tree: &'a SyntaxTree) -> Self {
        Self { id, node, tree }
    }

    pub fn root(&self) -> Result<Self, SyntaxError> {
        self.tree.root()
    }

    pub fn file_id(&self) -> FileId {
        self.tree.file_id
    }

    pub fn position(&self) -> Position {
        Position::new(self.tree.file_id, self.node.span)
    }

    pub fn child_count(&self) -> usize {
        match self.node.children_kind {
            SyntaxChildrenKind::None => 0,
            SyntaxChildrenKind::Single => 1,
            SyntaxChildrenKind::Binary => 2,
            SyntaxChildrenKind::ThreeOrMore => {
                (self.node.children.right - self.node.children.left) as usize
            }
        }
    }

    pub fn has_children(&self) -> bool {
        self.node.children_kind != SyntaxChildrenKind::None
    }

    pub fn has_left_child(&self) -> bool {
        matches!(
            self.node.children_kind,
            SyntaxChildrenKind::Single
                | SyntaxChildrenKind::Binary
                | SyntaxChildrenKind::ThreeOrMore
        )
    }

    pub fn has_right_child(&self) -> bool {
        matches!(
            self.node.children_kind,
            SyntaxChildrenKind::Binary | SyntaxChildrenKind::ThreeOrMore
        )
    }

    pub fn single_child(&self) -> Result<Self, SyntaxError> {
        debug_assert!(self.node.children_kind == SyntaxChildrenKind::Single);

        let left_id = self.node.children.left_id();
        let left = self.tree.read_node(left_id)?;

        Ok(left)
    }

    pub fn binary_children(&self) -> Result<(Self, Self), SyntaxError> {
        debug_assert!(self.node.children_kind == SyntaxChildrenKind::Binary);

        let left_id = self.node.children.left_id();
        let right_id = self.node.children.right_id();

        let left_child = self.tree.read_node(left_id)?;
        let right_child = self.tree.read_node(right_id)?;

        Ok((left_child, right_child))
    }

    pub fn children(self) -> SyntaxIterator<'a> {
        SyntaxIterator {
            parent: self,
            current_index: 0,
        }
    }

    pub fn child_pairs(self) -> SyntaxPairIterator<'a> {
        SyntaxPairIterator {
            parent: self,
            current_index: 0,
        }
    }

    pub fn as_component<T: SyntaxComponent<'a>>(&'a self) -> Result<T, SyntaxError> {
        T::from_reader(self)
    }

    pub fn draw_text_tree(&self, buffer: &mut String) {
        let mut ancestors = Vec::new();

        self.draw_text_tree_line(buffer, &mut ancestors, true);
    }

    fn draw_text_tree_line(&self, buffer: &mut String, ancestors: &mut Vec<bool>, is_last: bool) {
        for ancestor_is_last in &*ancestors {
            let indent = if *ancestor_is_last { "    " } else { "│   " };

            buffer.push_str(indent);
        }

        let connector = if self.node.kind == SyntaxKind::Root {
            "•"
        } else if is_last {
            "└─ "
        } else {
            "├─ "
        };

        buffer.push_str(connector);
        buffer.push_str(self.node.kind.as_str());

        if let Some(info) = self.node.flags.info_display(self.node.kind) {
            buffer.push_str(" (");
            buffer.push_str(info);
            buffer.push_str(")");
        }

        buffer.push('\n');

        let child_count = self.child_count();

        ancestors.push(is_last);

        if child_count == 0 {
            ancestors.pop();

            return;
        }

        let mut children = self.children().collect::<Vec<SyntaxReader>>();

        children.sort_by_key(|child| child.node.span);

        for (index, child) in children.into_iter().enumerate() {
            let child_is_last = index == child_count.saturating_sub(1);

            child.draw_text_tree_line(buffer, ancestors, child_is_last);
        }

        ancestors.pop();
    }
}

#[derive(Debug)]
pub struct SyntaxIterator<'a> {
    parent: SyntaxReader<'a>,
    current_index: usize,
}

impl<'a> SyntaxIterator<'a> {
    pub fn expect_next(&mut self) -> Result<SyntaxReader<'a>, SyntaxError> {
        self.next().ok_or_else(|| SyntaxError::MissingSyntaxChild {
            missing_index: self.current_index as u32,
            total_children: self.parent.child_count() as u32,
        })
    }
}

impl<'a> Iterator for SyntaxIterator<'a> {
    type Item = SyntaxReader<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = match self.parent.node.children_kind {
            SyntaxChildrenKind::Single | SyntaxChildrenKind::Binary if self.current_index == 0 => {
                self.parent.node.children.left_id()
            }
            SyntaxChildrenKind::Binary if self.current_index == 1 => {
                self.parent.node.children.right_id()
            }
            SyntaxChildrenKind::ThreeOrMore => {
                let start = self.parent.node.children.left as usize;
                let end = self.parent.node.children.right as usize;
                let index = start + self.current_index;

                if index >= end {
                    return None;
                }

                self.parent.tree.children[index]
            }
            _ => return None,
        };
        let node = self.parent.tree.nodes[id.0 as usize];
        self.current_index += 1;

        Some(SyntaxReader::new(id, node, self.parent.tree))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let start_length = self.parent.child_count();
        let remaining = start_length.saturating_sub(self.current_index);

        (remaining, Some(remaining))
    }
}

impl DoubleEndedIterator for SyntaxIterator<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let child_id = match self.parent.node.children_kind {
            SyntaxChildrenKind::Single if self.current_index == 0 => {
                let child_id = self.parent.node.children.left_id();
                self.current_index += 1;

                child_id
            }
            SyntaxChildrenKind::Binary if self.current_index < 2 => {
                let child_id = if self.current_index == 0 {
                    self.parent.node.children.left_id()
                } else {
                    self.parent.node.children.right_id()
                };
                self.current_index += 1;

                child_id
            }
            SyntaxChildrenKind::ThreeOrMore if self.current_index < self.parent.child_count() => {
                let child_index = self.parent.node.children.right as usize - self.current_index - 1;
                self.current_index += 1;

                self.parent.tree.children[child_index]
            }
            _ => return None,
        };

        self.parent.tree.read_node(child_id).ok()
    }
}

impl ExactSizeIterator for SyntaxIterator<'_> {}

pub struct SyntaxPairIterator<'a> {
    parent: SyntaxReader<'a>,
    current_index: usize,
}

impl<'a> SyntaxPairIterator<'a> {
    pub fn expect_next_pair(
        &mut self,
    ) -> Result<(SyntaxReader<'a>, SyntaxReader<'a>), SyntaxError> {
        self.next().ok_or_else(|| SyntaxError::MissingSyntaxChild {
            missing_index: self.current_index as u32,
            total_children: self.parent.child_count() as u32,
        })
    }
}

impl<'a> Iterator for SyntaxPairIterator<'a> {
    type Item = (SyntaxReader<'a>, SyntaxReader<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let (left_id, right_id) = match self.parent.node.children_kind {
            SyntaxChildrenKind::Binary if self.current_index == 0 => (
                self.parent.node.children.left_id(),
                self.parent.node.children.right_id(),
            ),
            SyntaxChildrenKind::ThreeOrMore => {
                let start = self.parent.node.children.left as usize;
                let left_index = start + self.current_index;
                let right_index = left_index + 1;

                if right_index >= self.parent.node.children.right as usize {
                    return None;
                }

                (
                    self.parent.tree.children[left_index],
                    self.parent.tree.children[right_index],
                )
            }
            _ => return None,
        };
        let left_node = self.parent.tree.nodes[left_id.0 as usize];
        let right_node = self.parent.tree.nodes[right_id.0 as usize];
        self.current_index += 2;

        Some((
            SyntaxReader::new(left_id, left_node, self.parent.tree),
            SyntaxReader::new(right_id, right_node, self.parent.tree),
        ))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let start_length = self.parent.child_count() / 2;
        let remaining = start_length.saturating_sub(self.current_index / 2);

        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for SyntaxPairIterator<'_> {}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    #[test]
    fn double_ended_iterator() {
        let (syntax_tree, errors) = parse(
            "fn main() -> i32 { 1 + 2 * 3 } fn foo() -> i32 { 4 - 5 / 6 } fn bar() -> i32 { 7 % 8 }",
        );

        assert!(errors.is_empty());

        let root = syntax_tree.root().unwrap();
        let mut forward_tree = root.children().collect::<Vec<_>>().into_iter();
        let mut backward_tree = root.children().rev().collect::<Vec<_>>().into_iter();

        assert_eq!(forward_tree.len(), backward_tree.len());

        loop {
            let forward = forward_tree.next();
            let backward = backward_tree.next_back();

            match (forward, backward) {
                (Some(forward), Some(backward)) => {
                    assert_eq!(forward.id, backward.id);
                    assert_eq!(forward.node.kind, backward.node.kind);
                }
                (None, None) => break,
                _ => panic!(),
            }
        }
    }
}
