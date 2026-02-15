use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use tracing::error;

use crate::{
    source::SourceFileId,
    syntax::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxPayload, SyntaxReader},
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

    pub fn is_module(&self) -> bool {
        self.nodes.first().is_some_and(|node| {
            matches!(
                node.kind,
                SyntaxKind::ModuleItem | SyntaxKind::PublicModuleItem
            )
        })
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn last_node_id(&self) -> SyntaxId {
        let index = self.nodes.len().saturating_sub(1) as u32;

        SyntaxId(index)
    }

    pub fn last_node(&self) -> Option<&SyntaxNode> {
        self.nodes.last()
    }

    pub fn last(&self) -> Option<(SyntaxId, &SyntaxNode)> {
        let id = self.last_node_id();

        self.last_node().map(|node| (id, node))
    }

    pub fn push(&mut self, node: SyntaxNode) -> SyntaxId {
        let index = self.nodes.len() as u32;

        self.nodes.push(node);

        SyntaxId(index)
    }

    pub fn pop(&mut self) -> Option<SyntaxNode> {
        self.nodes.pop()
    }

    pub fn replace_node(&mut self, id: SyntaxId, node: SyntaxNode) {
        if let Some(existing_node) = self.nodes.get_mut(id.0 as usize) {
            *existing_node = node;
        }
    }

    pub fn root(&self) -> Option<SyntaxReader<'_>> {
        let root_node = self.nodes.first()?;

        Some(SyntaxReader::new(SyntaxId::ROOT, root_node, self))
    }

    pub fn read_node(&self, id: SyntaxId) -> Option<SyntaxReader<'_>> {
        let node = self.get_node(id)?;

        Some(SyntaxReader::new(id, node, self))
    }

    pub fn get_node(&self, id: SyntaxId) -> Option<&SyntaxNode> {
        if id.is_none() {
            return None;
        }

        self.nodes.get(id.0 as usize)
    }

    pub fn get_children(&self, payload: SyntaxPayload) -> &[SyntaxId] {
        if payload.left_id().is_none() || payload.right_id().is_none() {
            return &[];
        }

        let child_range = payload.as_usize_range();

        if child_range.end > self.children.len() {
            error!(
                "Failed to get syntax nodes: invalid range {}..{} with length {}",
                child_range.start,
                child_range.end,
                self.children.len()
            );

            &[]
        } else {
            &self.children[child_range]
        }
    }

    pub fn add_children(&mut self, children: SmallVec<[SyntaxId; 4]>) -> SyntaxPayload {
        let start = self.children.len() as u32;
        let count = children.len() as u32;

        self.children.extend(children);

        SyntaxPayload::child_indices(start, count)
    }

    pub fn sorted_nodes(&self) -> Vec<SyntaxNode> {
        let mut nodes = self.nodes.clone();

        nodes.sort_by_key(|node| node.span.start());

        nodes
    }

    fn as_text_tree(&self) -> String {
        let root = match self.root() {
            Some(root) => root,
            None => return "<empty>".to_string(),
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
