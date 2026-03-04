use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    dust_error::{ErrorKind, InternalError},
    source::SourceFileId,
    syntax::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxReader},
};

/// A parsed Dust source code file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyntaxTree {
    pub file_id: SourceFileId,

    /// Append-only list of syntax nodes. Each node's ID is its index in this list.
    nodes: Vec<SyntaxNode>,

    /// Concatenated list of node indexes that represent children for nodes with more than two
    /// children.
    pub(crate) children: Vec<SyntaxId>,
}

impl SyntaxTree {
    pub fn new(file_id: SourceFileId) -> Self {
        Self {
            file_id,
            nodes: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn is_root(&self) -> bool {
        self.nodes
            .first()
            .is_some_and(|node| node.kind == SyntaxKind::Root)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn root(&self) -> Result<SyntaxReader<'_>, ErrorKind> {
        let root_node =
            self.nodes
                .first()
                .ok_or(ErrorKind::Internal(InternalError::MissingSyntaxNode(
                    SyntaxId::ROOT,
                )))?;

        Ok(SyntaxReader::new(SyntaxId::ROOT, root_node, self))
    }

    pub fn get_node(&self, id: SyntaxId) -> Result<SyntaxReader<'_>, ErrorKind> {
        let node = self
            .nodes
            .get(id.0 as usize)
            .ok_or(ErrorKind::Internal(InternalError::MissingSyntaxNode(id)))?;

        Ok(SyntaxReader::new(id, node, self))
    }

    pub fn iter(&self) -> impl Iterator<Item = SyntaxReader<'_>> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| SyntaxReader::new(SyntaxId(index as u32), node, self))
    }

    pub fn sorted_nodes(&self) -> Vec<SyntaxNode> {
        fn collect_depth_first(node: SyntaxReader, nodes: &mut Vec<SyntaxNode>) {
            nodes.push(*node.inner());

            let children = match node.children() {
                Ok(children) => children,
                Err(error) => {
                    error!("{}", error.into_internal());

                    return;
                }
            };

            for child in children {
                collect_depth_first(child, nodes);
            }
        }

        let root = match self.root() {
            Ok(root) => root,
            Err(_) => return Vec::new(),
        };
        let mut nodes = Vec::with_capacity(self.nodes.len());

        collect_depth_first(root, &mut nodes);

        nodes
    }

    fn as_text_tree(&self) -> String {
        let root = match self.root() {
            Ok(root) => root,
            Err(_) => return "<empty>".to_string(),
        };
        let mut buffer = String::new();

        root.draw_text_tree(&mut buffer);

        buffer
    }
}

impl Display for SyntaxTree {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Syntax Tree: {} nodes\n{}",
            self.node_count(),
            self.as_text_tree()
        )
    }
}

pub struct SyntaxTreeBuilder {
    tree: SyntaxTree,
}

impl SyntaxTreeBuilder {
    pub fn new(file_id: SourceFileId) -> Self {
        Self {
            tree: SyntaxTree::new(file_id),
        }
    }

    pub fn file_id(&self) -> SourceFileId {
        self.tree.file_id
    }

    pub fn add_node(&mut self, node: SyntaxNode) -> SyntaxId {
        let id = SyntaxId(self.tree.nodes.len() as u32);

        self.tree.nodes.push(node);

        id
    }

    pub fn replace_node(&mut self, id: SyntaxId, node: SyntaxNode) {
        self.tree.nodes[id.0 as usize] = node;
    }

    pub fn add_children(&mut self, children: &[SyntaxId]) -> (u32, u32) {
        let start_index = self.tree.children.len() as u32;
        let length = children.len() as u32;

        self.tree.children.extend_from_slice(children);

        (start_index, length)
    }

    pub fn build(self) -> SyntaxTree {
        self.tree
    }
}
