use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    source::SourceFileId,
    syntax::{
        SyntaxId,
        error::SyntaxError,
        node::{SyntaxKind, SyntaxNode, SyntaxPayload},
        reader::SyntaxReader,
    },
};

/// A parsed Dust source code file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyntaxTree {
    pub file_id: SourceFileId,

    /// Append-only list of syntax nodes. Each node's ID is its index in this list.
    pub(super) nodes: Vec<SyntaxNode>,

    /// Concatenated list of node IDs for nodes with more than two children.
    pub(super) children: Vec<SyntaxId>,
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

    pub fn root(&self) -> Result<SyntaxReader<'_>, SyntaxError> {
        let root_node = self
            .nodes
            .first()
            .ok_or(SyntaxError::MissingSyntaxNode(SyntaxId::ROOT))?;

        Ok(SyntaxReader::new(SyntaxId::ROOT, root_node, self))
    }

    pub fn get_node(&self, id: SyntaxId) -> Result<SyntaxReader<'_>, SyntaxError> {
        let node = self
            .nodes
            .get(id.0 as usize)
            .ok_or(SyntaxError::MissingSyntaxNode(id))?;

        Ok(SyntaxReader::new(id, node, self))
    }

    pub fn iter(&self) -> impl Iterator<Item = SyntaxReader<'_>> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| SyntaxReader::new(SyntaxId(index as u32), node, self))
    }

    pub fn sorted_nodes(&self) -> Vec<SyntaxNode> {
        fn collect_depth_first(reader: SyntaxReader, nodes: &mut Vec<SyntaxNode>) {
            nodes.push(*reader.node);

            for child in reader.children() {
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
}

impl Display for SyntaxTree {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let root = match self.root() {
            Ok(root) => root,
            Err(_) => return write!(f, "Syntax Tree: <empty>"),
        };
        let mut buffer = String::new();

        root.draw_text_tree(&mut buffer);

        write!(f, "Syntax Tree: {} nodes\n{buffer}", self.node_count())
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

    pub fn add_children<T>(
        &mut self,
        children: impl IntoIterator<Item = SyntaxId>,
    ) -> SyntaxPayload {
        let start_index = self.tree.children.len() as u32;

        self.tree.children.extend(children);

        let end_index = self.tree.children.len() as u32;

        SyntaxPayload {
            left: start_index,
            right: end_index,
        }
    }

    pub fn build(self) -> SyntaxTree {
        self.tree
    }
}
