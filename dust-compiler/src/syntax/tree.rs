use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    source::CodeId,
    syntax::{
        SyntaxId,
        error::SyntaxError,
        node::{SyntaxChildren, SyntaxNode},
        reader::SyntaxReader,
    },
};

/// Parsed Dust source code.
#[derive(Clone, Serialize, Deserialize)]
pub struct SyntaxTree {
    pub code_id: CodeId,

    /// Append-only list of syntax nodes. Each node's ID is its index in this list.
    pub(super) nodes: Vec<SyntaxNode>,

    /// Concatenated list of node IDs for nodes with more than two children.
    pub(super) children: Vec<SyntaxId>,
}

impl SyntaxTree {
    pub fn new(code_id: CodeId) -> Self {
        Self {
            code_id,
            nodes: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn read_root(&self) -> Result<SyntaxReader<'_>, SyntaxError> {
        self.read_node(SyntaxId::ROOT)
    }

    pub fn read_node(&self, id: SyntaxId) -> Result<SyntaxReader<'_>, SyntaxError> {
        let node = self
            .nodes
            .get(id.0 as usize)
            .ok_or(SyntaxError::MissingNode(id))?;

        Ok(SyntaxReader::new(id, *node, self))
    }

    pub fn sort_nodes(&self) -> Vec<SyntaxNode> {
        fn collect_depth_first(reader: SyntaxReader, nodes: &mut Vec<SyntaxNode>) {
            nodes.push(reader.node);

            for child in reader.children() {
                collect_depth_first(child, nodes);
            }
        }

        let root = match self.read_root() {
            Ok(root) => root,
            Err(_) => return Vec::new(),
        };
        let mut nodes = Vec::with_capacity(self.nodes.len());

        collect_depth_first(root, &mut nodes);

        nodes
    }
}

impl Debug for SyntaxTree {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "SyntaxTree {{ code_id: {:?}, nodes: {}, children: {} }}",
            self.code_id,
            self.nodes.len(),
            self.children.len()
        )
    }
}

impl Display for SyntaxTree {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let root = match self.read_root() {
            Ok(root) => root,
            Err(_) => return write!(f, "Syntax Tree: <empty>"),
        };
        let mut buffer = String::new();

        root.draw_text_tree(&mut buffer);

        write!(f, "Syntax Tree: {} nodes\n{buffer}", self.nodes.len())
    }
}

pub struct SyntaxTreeBuilder {
    pub code_id: CodeId,
    nodes: Vec<SyntaxNode>,
    children: Vec<SyntaxId>,
}

impl SyntaxTreeBuilder {
    pub fn new(code_id: CodeId) -> Self {
        Self {
            code_id,
            nodes: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn build(self) -> SyntaxTree {
        SyntaxTree {
            code_id: self.code_id,
            nodes: self.nodes,
            children: self.children,
        }
    }

    pub fn add_node(&mut self, node: SyntaxNode) -> SyntaxId {
        let id = SyntaxId(self.nodes.len() as u32);

        self.nodes.push(node);

        id
    }

    pub fn replace_node(&mut self, id: SyntaxId, node: SyntaxNode) {
        self.nodes[id.0 as usize] = node;
    }

    pub fn add_children(&mut self, children: impl IntoIterator<Item = SyntaxId>) -> SyntaxChildren {
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
}
