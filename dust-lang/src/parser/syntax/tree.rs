use std::fmt::{self, Display, Formatter};

use termtree::Tree;
use tracing::error;

use crate::{
    parser::syntax::{
        SyntaxId, SyntaxKind, SyntaxNode, SyntaxNodeChildren, SyntaxPayload, SyntaxReader,
    },
    source::SourceFileId,
};

/// Lossless abstract syntax tree representing a Dust source code file.
#[derive(Clone, Debug)]
pub struct SyntaxTree {
    pub file_id: SourceFileId,

    /// Append-only list of syntax nodes. Each node's ID is its index in this list.
    pub(crate) nodes: Vec<SyntaxNode>,

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

    pub fn is_main_function(&self) -> bool {
        self.nodes
            .first()
            .is_some_and(|node| node.kind == SyntaxKind::MainFunctionItem)
    }

    pub fn is_module(&self) -> bool {
        self.nodes
            .first()
            .is_some_and(|node| node.kind == SyntaxKind::ModuleItem)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
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

    pub fn add_node(&mut self, node: SyntaxNode) -> SyntaxId {
        let index = self.nodes.len() as u32;

        self.nodes.push(node);

        SyntaxId(index)
    }

    pub fn root(&self) -> Option<SyntaxReader<'_>> {
        let root_node = self.nodes.first()?;

        Some(SyntaxReader::new(SyntaxId::ROOT, root_node, self))
    }

    pub fn get_node(&self, id: SyntaxId) -> Option<&SyntaxNode> {
        if id == SyntaxId::NONE {
            return None;
        }

        self.nodes.get(id.0 as usize)
    }

    pub fn get_children(&self, payload: SyntaxPayload) -> &[SyntaxId] {
        if payload.left_id() == SyntaxId::NONE || payload.right_id() == SyntaxId::NONE {
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

    pub fn add_children(&mut self, children: &[SyntaxId]) -> SyntaxPayload {
        if children.is_empty() {
            SyntaxPayload::empty()
        } else {
            let payload = SyntaxPayload::child_indices(self.children.len(), children.len());

            self.children.extend_from_slice(children);

            payload
        }
    }

    pub fn sorted_nodes(&self) -> Vec<SyntaxNode> {
        let mut nodes = self.nodes.clone();

        nodes.sort_by_key(|node| node.span.0);

        nodes
    }

    fn as_text_tree(&self) -> String {
        fn build_text_tree<'a>(
            leaf_id: SyntaxId,
            parent_tree: Option<&mut Tree<&'a SyntaxNode>>,
            syntax_tree: &'a SyntaxTree,
        ) -> Option<Tree<&'a SyntaxNode>> {
            if leaf_id == SyntaxId::NONE {
                return None;
            }

            let node = match syntax_tree.get_node(leaf_id) {
                Some(node) => node,
                None => {
                    error!("Failed to build text tree: missing syntax node with ID {leaf_id:?}");

                    return parent_tree.cloned();
                }
            };
            let mut leaf = Tree::new(node);

            match node.children() {
                SyntaxNodeChildren::None => {}
                SyntaxNodeChildren::Single(id) => {
                    build_text_tree(id, Some(&mut leaf), syntax_tree);
                }
                SyntaxNodeChildren::Binary(left, right) => {
                    build_text_tree(left, Some(&mut leaf), syntax_tree);
                    build_text_tree(right, Some(&mut leaf), syntax_tree);
                }
                SyntaxNodeChildren::Multiple(payload) => {
                    let child_ids = syntax_tree.get_children(payload);

                    for child_id in child_ids {
                        build_text_tree(*child_id, Some(&mut leaf), syntax_tree);
                    }
                }
            }

            if let Some(parent_tree) = parent_tree {
                parent_tree.leaves.push(leaf);

                None
            } else {
                Some(leaf)
            }
        }

        build_text_tree(SyntaxId::ROOT, None, self)
            .map(|tree| tree.to_string())
            .unwrap_or_else(|| "<empty>".to_string())
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
