use std::fmt::{self, Display, Formatter};

use termtree::Tree;
use tracing::error;

use crate::{
    source::SourceFileId,
    syntax::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxNodeChildren, SyntaxPayload, SyntaxReader},
};

/// Lossless abstract syntax tree representing a Dust source code file.
#[derive(Debug)]
pub struct SyntaxTree {
    pub file_id: SourceFileId,

    /// List of nodes in the tree in the order they were parsed according to the Pratt algorithm
    /// used by the parser.
    pub nodes: Vec<SyntaxNode>,

    /// Concatenated list of node indexes that represent children for nodes whose child indexes
    /// cannot be stored directly in the node (i.e. blocks and the root node).
    pub children: Vec<SyntaxId>,
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

    pub fn get_children(&self, payload: SyntaxPayload) -> Option<&[SyntaxId]> {
        let start_index = payload.left as usize;
        let count = payload.right as usize;

        self.children.get(start_index..start_index + count)
    }

    pub fn add_children(&mut self, children: &[SyntaxId]) -> SyntaxPayload {
        let payload = SyntaxPayload {
            left: self.children.len() as u32,
            right: children.len() as u32,
        };

        self.children.extend_from_slice(children);

        payload
    }

    pub fn sorted_nodes(&self) -> Vec<SyntaxNode> {
        let mut nodes = self.nodes.clone();

        nodes.sort_by_key(|node| node.span.0);

        nodes
    }

    fn as_text_tree(&self) -> String {
        fn build_tree(
            parent: &mut Tree<SyntaxNode>,
            current_child_id: SyntaxId,
            syntax_tree: &SyntaxTree,
        ) {
            if current_child_id == SyntaxId::NONE {
                return;
            }

            let current_child = &syntax_tree.nodes[current_child_id.0 as usize];
            let mut leaf = Tree::new(*current_child);

            match current_child.children() {
                SyntaxNodeChildren::None => {}
                SyntaxNodeChildren::Single(syntax_id) => {
                    build_tree(&mut leaf, syntax_id, syntax_tree);
                }
                SyntaxNodeChildren::Binary(left, right) => {
                    build_tree(&mut leaf, left, syntax_tree);
                    build_tree(&mut leaf, right, syntax_tree);
                }
                SyntaxNodeChildren::Multiple(children) => {
                    for child_id in syntax_tree.get_children(children).unwrap_or_else(|| {
                        error!(
                            "Failed to get {} syntax nodes starting at index {}",
                            children.right, children.left
                        );

                        &[]
                    }) {
                        build_tree(&mut leaf, *child_id, syntax_tree);
                    }
                }
            }

            // Prevent displaying the root node twice
            if leaf.root == parent.root {
                parent.leaves.extend(leaf.leaves);
            } else {
                parent.leaves.push(leaf);
            }
        }

        let top_node = match self.root() {
            Some(node) => node,
            None => return "<empty>".to_string(),
        };
        let mut root = Tree::new(*top_node.inner());

        build_tree(&mut root, SyntaxId(0), self);
        root.to_string()
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
