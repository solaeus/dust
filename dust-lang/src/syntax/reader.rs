use tracing::error;

use crate::{
    dust_error::{ErrorKind, InternalError},
    source::{Position, SourceFileId, Span},
    syntax::{
        SyntaxId, SyntaxKind, SyntaxNode, SyntaxPayload, SyntaxTree, node::SyntaxPayloadKind,
    },
};

#[derive(Clone, Copy, Debug)]
pub struct SyntaxReader<'a> {
    pub id: SyntaxId,
    tree: &'a SyntaxTree,
    node: &'a SyntaxNode,
}

impl<'a> SyntaxReader<'a> {
    pub fn new(id: SyntaxId, node: &'a SyntaxNode, tree: &'a SyntaxTree) -> Self {
        Self { id, node, tree }
    }

    pub fn root(&self) -> Result<Self, ErrorKind> {
        self.tree.root()
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
        matches!(
            self.node.payload_kind,
            SyntaxPayloadKind::SingleChild
                | SyntaxPayloadKind::BinaryChildren
                | SyntaxPayloadKind::MultipleChildren
        )
    }

    pub fn has_right_child(&self) -> bool {
        matches!(
            self.node.payload_kind,
            SyntaxPayloadKind::BinaryChildren | SyntaxPayloadKind::MultipleChildren
        )
    }

    pub fn child(&self) -> Result<Self, ErrorKind> {
        if self.node.payload_kind != SyntaxPayloadKind::SingleChild {
            return Err(ErrorKind::Internal(InternalError::ExpectedSyntaxChildren {
                expected: 1,
                actual: self.child_count(),
            }));
        }

        let left_id = self.node.payload.left_id();
        let left = self.tree.get_node(left_id)?;

        Ok(left)
    }

    pub fn binary_children(&self) -> Result<(Self, Self), ErrorKind> {
        if self.node.payload_kind != SyntaxPayloadKind::BinaryChildren {
            return Err(ErrorKind::Internal(InternalError::ExpectedSyntaxChildren {
                expected: 2,
                actual: self.child_count(),
            }));
        }

        let left_id = self.node.payload.left_id();
        let right_id = self.node.payload.right_id();

        let left_child = self.tree.get_node(left_id)?;
        let right_child = self.tree.get_node(right_id)?;

        Ok((left_child, right_child))
    }

    pub fn children(&'a self) -> Result<SyntaxReaderIterator<'a>, ErrorKind> {
        match self.node.payload_kind {
            SyntaxPayloadKind::Empty => {}
            SyntaxPayloadKind::Value => {
                return Err(ErrorKind::Internal(InternalError::InvalidSyntaxPayload(
                    self.payload(),
                )));
            }
            SyntaxPayloadKind::SingleChild => {
                if self.node.payload.left >= self.tree.node_count() as u32 {
                    return Err(ErrorKind::Internal(InternalError::MissingSyntaxNode(
                        self.payload().left_id(),
                    )));
                }
            }
            SyntaxPayloadKind::BinaryChildren => {
                if self.node.payload.left >= self.tree.node_count() as u32 {
                    return Err(ErrorKind::Internal(InternalError::MissingSyntaxNode(
                        self.payload().left_id(),
                    )));
                }

                if self.node.payload.right >= self.tree.node_count() as u32 {
                    return Err(ErrorKind::Internal(InternalError::MissingSyntaxNode(
                        self.payload().right_id(),
                    )));
                }
            }
            SyntaxPayloadKind::MultipleChildren => {
                if self.node.payload.right >= self.tree.children.len() as u32 {
                    return Err(ErrorKind::Internal(InternalError::InvalidSyntaxPayload(
                        self.payload(),
                    )));
                }
            }
        }

        Ok(SyntaxReaderIterator::new(self))
    }

    pub fn last_child(&'a self) -> Result<Option<Self>, ErrorKind> {
        match self.node.payload_kind {
            SyntaxPayloadKind::SingleChild => self.child().map(Some),
            SyntaxPayloadKind::BinaryChildren => {
                let right_id = self.node.payload.right_id();

                self.tree.get_node(right_id).map(Some)
            }
            SyntaxPayloadKind::MultipleChildren => Ok(SyntaxReaderIterator::new(self).last()),
            _ => Ok(None),
        }
    }

    pub fn draw_text_tree(&self, buffer: &mut String) {
        let mut ancestors = Vec::new();

        self.draw_text_tree_line(buffer, &mut ancestors, false);
    }

    fn draw_text_tree_line(&self, buffer: &mut String, ancestors: &mut Vec<bool>, is_last: bool) {
        let children = match self.children() {
            Ok(children) => children,
            Err(error) => {
                error!("{}", error.into_internal());

                return;
            }
        };

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

        let size = children.len();

        ancestors.push(is_last);

        for (index, child) in children.enumerate() {
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
        match self.parent.node.payload_kind {
            SyntaxPayloadKind::Empty | SyntaxPayloadKind::Value => true,
            SyntaxPayloadKind::SingleChild
            | SyntaxPayloadKind::BinaryChildren
            | SyntaxPayloadKind::MultipleChildren => false,
        }
    }

    pub fn expect_next(&mut self) -> Result<SyntaxReader<'a>, ErrorKind> {
        self.next().ok_or_else(|| {
            ErrorKind::Internal(InternalError::MissingSyntaxChild {
                total_children: self.parent.child_count(),
            })
        })
    }
}

impl<'a> Iterator for SyntaxReaderIterator<'a> {
    type Item = SyntaxReader<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let child_id = match self.parent.node.payload_kind {
            SyntaxPayloadKind::SingleChild if self.current_index == 0 => {
                let child_id = self.parent.node.payload.left_id();
                self.current_index += 1;

                child_id
            }
            SyntaxPayloadKind::BinaryChildren if self.current_index < 2 => {
                let child_id = if self.current_index == 0 {
                    self.parent.node.payload.left_id()
                } else {
                    self.parent.node.payload.right_id()
                };
                self.current_index += 1;

                child_id
            }
            SyntaxPayloadKind::MultipleChildren if self.current_index < self.len() => {
                let child_index = self.parent.payload().as_usize_range().start + self.current_index;
                self.current_index += 1;

                self.parent.tree.children[child_index]
            }
            _ => return None,
        };

        self.parent.tree.get_node(child_id).ok()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let start_length = self.parent.child_count();
        let remaining = start_length.saturating_sub(self.current_index);

        (remaining, Some(remaining))
    }
}

impl DoubleEndedIterator for SyntaxReaderIterator<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let child_id = match self.parent.node.payload_kind {
            SyntaxPayloadKind::SingleChild if self.current_index == 0 => {
                let child_id = self.parent.node.payload.left_id();
                self.current_index += 1;

                child_id
            }
            SyntaxPayloadKind::BinaryChildren if self.current_index < 2 => {
                let child_id = if self.current_index == 0 {
                    self.parent.node.payload.left_id()
                } else {
                    self.parent.node.payload.right_id()
                };
                self.current_index += 1;

                child_id
            }
            SyntaxPayloadKind::MultipleChildren if self.current_index < self.len() => {
                let child_index =
                    self.parent.payload().as_usize_range().end - 1 - self.current_index;
                self.current_index += 1;

                self.parent.tree.children[child_index]
            }
            _ => return None,
        };

        self.parent.tree.get_node(child_id).ok()
    }
}

impl ExactSizeIterator for SyntaxReaderIterator<'_> {}
