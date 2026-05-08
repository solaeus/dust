use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    source::SourceCodeId,
    syntax::{
        SyntaxId,
        error::SyntaxError,
        node::{SyntaxChildren, SyntaxNode},
        reader::SyntaxReader,
    },
};

/// Parsed Dust source code.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyntaxTree {
    pub source_id: SourceCodeId,

    /// Append-only list of syntax nodes. Each node's ID is its index in this list.
    pub(super) nodes: Vec<SyntaxNode>,

    /// Concatenated list of node IDs for nodes with more than two children.
    pub(super) children: Vec<SyntaxId>,
}

impl SyntaxTree {
    pub fn new(source_id: SourceCodeId) -> Self {
        Self {
            source_id,
            nodes: Vec::new(),
            children: Vec::new(),
        }
    }

    pub(crate) fn placeholder() -> Self {
        Self {
            source_id: SourceCodeId::MAIN,
            nodes: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn root(&self) -> Result<SyntaxReader<'_>, SyntaxError> {
        let root_node = self
            .nodes
            .first()
            .ok_or(SyntaxError::MissingNode(SyntaxId::ROOT))?;

        Ok(SyntaxReader::new(SyntaxId::ROOT, *root_node, self))
    }

    pub(crate) fn add_node(&mut self, node: SyntaxNode) -> SyntaxId {
        let id = SyntaxId(self.nodes.len() as u32);

        self.nodes.push(node);

        id
    }

    pub(crate) fn add_children(
        &mut self,
        children: impl IntoIterator<Item = SyntaxId>,
    ) -> SyntaxChildren {
        let left = self.children.len() as u32;

        self.children.extend(children);

        let right = self.children.len() as u32;

        debug_assert!(
            right - left > 2,
            "SyntaxTree::add_children should only be used for 3 or more children. Nodes with 2 or \
            fewer children can encode their IDs directly in the node."
        );

        SyntaxChildren { left, right }
    }

    pub(crate) fn replace_node(&mut self, id: SyntaxId, node: SyntaxNode) {
        self.nodes[id.0 as usize] = node;
    }

    pub fn read_node(&self, id: SyntaxId) -> Result<SyntaxReader<'_>, SyntaxError> {
        let node = self
            .nodes
            .get(id.0 as usize)
            .ok_or(SyntaxError::MissingNode(id))?;

        Ok(SyntaxReader::new(id, *node, self))
    }

    pub fn iter(&self) -> impl Iterator<Item = SyntaxReader<'_>> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| SyntaxReader::new(SyntaxId(index as u32), *node, self))
    }

    pub fn sorted_nodes(&self) -> Vec<SyntaxNode> {
        fn collect_depth_first(reader: SyntaxReader, nodes: &mut Vec<SyntaxNode>) {
            nodes.push(reader.node);

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

        write!(f, "Syntax Tree: {} nodes\n{buffer}", self.nodes.len())
    }
}
