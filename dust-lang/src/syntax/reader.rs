use tracing::warn;

use crate::{
    dust_error::{DustError, InternalError},
    source::{Position, SourceFileId, Span},
    syntax::{
        SyntaxId, SyntaxKind, SyntaxNode, SyntaxPayload, SyntaxTree, node::SyntaxPayloadKind,
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

    pub fn root(&self) -> Result<Self, DustError> {
        self.tree
            .root()
            .ok_or(DustError::Internal(InternalError::MissingSyntaxNode(
                SyntaxId::ROOT,
            )))
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
        match self.node.payload_kind {
            SyntaxPayloadKind::SingleChild => 1,
            SyntaxPayloadKind::BinaryChildren => 2,
            SyntaxPayloadKind::MultipleChildren => self.node.payload.right as usize,
            _ => 0,
        }
    }

    pub fn has_left_child(&self) -> bool {
        self.node.payload.left_id() != SyntaxId::NONE
    }

    pub fn has_right_child(&self) -> bool {
        self.node.payload.right_id() != SyntaxId::NONE
    }

    pub fn left_child(&self) -> Result<Option<Self>, DustError> {
        let left_id = self.node.payload.left_id();

        if self.node.payload.left_id() == SyntaxId::NONE {
            Ok(None)
        } else {
            let child_node = self.tree.get_node(left_id).ok_or(DustError::Internal(
                InternalError::MissingSyntaxNode(left_id),
            ))?;

            Ok(Some(SyntaxReader::new(left_id, child_node, self.tree)))
        }
    }

    pub fn right_child(&self) -> Result<Option<Self>, DustError> {
        let right_id = self.node.payload.right_id();

        if self.node.payload.right_id() == SyntaxId::NONE {
            Ok(None)
        } else {
            let child_node = self.tree.get_node(right_id).ok_or(DustError::Internal(
                InternalError::MissingSyntaxNode(right_id),
            ))?;

            Ok(Some(SyntaxReader::new(right_id, child_node, self.tree)))
        }
    }

    pub fn expect_left_child(&self) -> Result<Self, DustError> {
        let left_id = self.node.payload.left_id();
        let left_node = self.tree.get_node(left_id).ok_or(DustError::Internal(
            InternalError::MissingSyntaxNode(left_id),
        ))?;

        Ok(SyntaxReader::new(left_id, left_node, self.tree))
    }

    pub fn expect_right_child(&self) -> Result<Self, DustError> {
        let right_id = self.node.payload.right_id();
        let right_node = self.tree.get_node(right_id).ok_or(DustError::Internal(
            InternalError::MissingSyntaxNode(right_id),
        ))?;

        Ok(SyntaxReader::new(right_id, right_node, self.tree))
    }

    pub fn expect_binary_children(&self) -> Result<(Self, Self), DustError> {
        let left_child = self.expect_left_child()?;
        let right_child = self.expect_right_child()?;

        Ok((left_child, right_child))
    }

    pub fn expect_multiple_children(&self) -> Result<SyntaxReaderIterator<'a>, DustError> {
        let child_ids = self.tree.get_child_ids(self.node.payload);

        if child_ids.is_empty() {
            return Err(DustError::Internal(InternalError::MissingSyntaxChildren(
                self.node.payload,
            )));
        }

        #[cfg(debug_assertions)]
        if child_ids.len() < 3 {
            warn!(
                "Using multiple_children for a node with fewer than 3 children:\nID: {:?}\nNode:{:#?}",
                self.id, self.node
            );
        }

        Ok(SyntaxReaderIterator::Multiple {
            child_ids,
            tree: self.tree,
            current_index: 0,
        })
    }

    pub fn children(&'a self) -> SyntaxReaderIterator<'a> {
        match self.node.payload_kind {
            SyntaxPayloadKind::SingleChild => SyntaxReaderIterator::Single {
                child_id: self.node.payload.left_id(),
                tree: self.tree,
                yielded: false,
            },
            SyntaxPayloadKind::BinaryChildren => SyntaxReaderIterator::Binary {
                left_child_id: self.node.payload.left_id(),
                right_child_id: self.node.payload.right_id(),
                tree: self.tree,
                current_index: 0,
            },
            SyntaxPayloadKind::MultipleChildren => SyntaxReaderIterator::Multiple {
                child_ids: self.tree.get_child_ids(self.node.payload),
                tree: self.tree,
                current_index: 0,
            },
            _ => SyntaxReaderIterator::Empty,
        }
    }

    pub fn last_child(&'a self) -> Option<Self> {
        self.children().next_back()
    }

    pub fn draw_text_tree(&self, buffer: &mut String) {
        buffer.push_str(self.node.kind.as_str());
        buffer.push('\n');

        let children = self.children();
        let size = children.len();
        let mut ancestors = Vec::new();

        for (index, child) in children.enumerate() {
            let is_last = index == size.saturating_sub(1);

            child.draw_text_tree_line(buffer, &mut ancestors, is_last);
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

        if self.node.payload_kind == SyntaxPayloadKind::Value {
            buffer.push_str(": ");

            match self.node.kind {
                SyntaxKind::BooleanExpression => {
                    let boolean = self.node.payload.decode_boolean();

                    buffer.push_str(&boolean.to_string());
                }
                SyntaxKind::ByteExpression => {
                    let byte = self.node.payload.decode_byte();

                    buffer.push_str(&byte.to_string());
                }
                SyntaxKind::CharacterExpression => {
                    let character = self.node.payload.decode_character();

                    buffer.push(character);
                }
                SyntaxKind::FloatExpression => {
                    let float = self.node.payload.decode_float();

                    buffer.push_str(&float.to_string());
                }
                SyntaxKind::IntegerExpression => {
                    let integer = self.node.payload.decode_integer();

                    buffer.push_str(&integer.to_string());
                }
                SyntaxKind::StringExpression => {
                    let string = self.node.payload.decode_string();

                    buffer.push_str(&string);
                }
                _ => {}
            }
        }

        buffer.push('\n');

        let children = self.children();
        let size = children.len();

        ancestors.push(is_last);

        for (index, child) in children.enumerate() {
            let child_is_last = index == size.saturating_sub(1);

            child.draw_text_tree_line(buffer, ancestors, child_is_last);
        }

        ancestors.pop();
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SyntaxReaderIterator<'a> {
    Empty,
    Single {
        child_id: SyntaxId,
        tree: &'a SyntaxTree,
        yielded: bool,
    },
    Binary {
        left_child_id: SyntaxId,
        right_child_id: SyntaxId,
        tree: &'a SyntaxTree,
        current_index: u8,
    },
    Multiple {
        child_ids: &'a [SyntaxId],
        tree: &'a SyntaxTree,
        current_index: usize,
    },
}

impl<'a> SyntaxReaderIterator<'a> {
    pub fn len(&self) -> usize {
        match self {
            SyntaxReaderIterator::Empty => 0,
            SyntaxReaderIterator::Single { .. } => 1,
            SyntaxReaderIterator::Binary { .. } => 2,
            SyntaxReaderIterator::Multiple { child_ids, .. } => child_ids.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            SyntaxReaderIterator::Empty => true,
            SyntaxReaderIterator::Single { yielded, .. } => *yielded,
            SyntaxReaderIterator::Binary { current_index, .. } => *current_index >= 2,
            SyntaxReaderIterator::Multiple { child_ids, .. } => child_ids.is_empty(),
        }
    }

    pub fn expect_next(&mut self) -> Result<SyntaxReader<'a>, DustError> {
        self.next().ok_or_else(|| {
            DustError::Internal(InternalError::MissingSyntaxChild {
                total_children: self.len(),
            })
        })
    }
}

impl<'a> Iterator for SyntaxReaderIterator<'a> {
    type Item = SyntaxReader<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SyntaxReaderIterator::Single {
                child_id,
                tree,
                yielded,
            } if !*yielded => {
                let child_node = tree.get_node(*child_id)?;
                *yielded = true;

                Some(SyntaxReader::new(*child_id, child_node, tree))
            }
            SyntaxReaderIterator::Binary {
                left_child_id,
                right_child_id,
                tree,
                current_index,
            } => {
                let child_id = match *current_index {
                    0 => *left_child_id,
                    1 => *right_child_id,
                    _ => return None,
                };
                let child_node = tree.get_node(child_id)?;
                *current_index += 1;

                Some(SyntaxReader::new(child_id, child_node, tree))
            }
            SyntaxReaderIterator::Multiple {
                child_ids,
                tree,
                current_index,
            } if *current_index < child_ids.len() => {
                let child_id = child_ids[*current_index];
                let child_node = tree.get_node(child_id)?;
                *current_index += 1;

                Some(SyntaxReader::new(child_id, child_node, tree))
            }
            _ => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = match self {
            SyntaxReaderIterator::Empty => 0,
            SyntaxReaderIterator::Single { yielded, .. } => {
                if *yielded {
                    0
                } else {
                    1
                }
            }
            SyntaxReaderIterator::Binary { current_index, .. } => {
                2_u8.saturating_sub(*current_index) as usize
            }
            SyntaxReaderIterator::Multiple {
                child_ids,
                current_index,
                ..
            } => child_ids.len().saturating_sub(*current_index),
        };

        (remaining, Some(remaining))
    }
}

impl<'a> DoubleEndedIterator for SyntaxReaderIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            SyntaxReaderIterator::Single {
                child_id,
                tree,
                yielded,
            } if !*yielded => {
                let child_node = tree.get_node(*child_id)?;
                *yielded = true;

                Some(SyntaxReader::new(*child_id, child_node, tree))
            }
            SyntaxReaderIterator::Binary {
                left_child_id,
                right_child_id,
                tree,
                current_index,
            } if *current_index < 2 => {
                let child_id = if *current_index == 0 {
                    *right_child_id
                } else {
                    *left_child_id
                };
                let child_node = tree.get_node(child_id)?;
                *current_index += 1;

                Some(SyntaxReader::new(child_id, child_node, tree))
            }
            SyntaxReaderIterator::Multiple {
                child_ids,
                tree,
                current_index,
            } if *current_index < child_ids.len() => {
                let child_id = child_ids[child_ids.len() - 1 - *current_index];
                let child_node = tree.get_node(child_id)?;
                *current_index += 1;

                Some(SyntaxReader::new(child_id, child_node, tree))
            }
            _ => None,
        }
    }
}

impl ExactSizeIterator for SyntaxReaderIterator<'_> {}
