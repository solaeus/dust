mod syntax_node;

use std::fmt::{self, Display, Formatter};

pub use syntax_node::{SyntaxKind, SyntaxNode, SyntaxNodeChildren};
use termtree::Tree;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxId(pub u32);

impl SyntaxId {
    pub const NONE: SyntaxId = SyntaxId(u32::MAX);

    pub fn is_none(&self) -> bool {
        *self == SyntaxId::NONE
    }
}

/// Lossless abstract syntax tree representing a Dust source code file.
#[derive(Debug)]
pub struct SyntaxTree {
    /// List of nodes in the tree in the order they were parsed according to the Pratt algorithm
    /// used by the parser.
    pub nodes: Vec<SyntaxNode>,

    /// Concatenated list of node indexes that represent children for nodes whose child indexes
    /// cannot be stored directly in the node (i.e. blocks and the root node).
    pub children: Vec<SyntaxId>,
}

impl SyntaxTree {
    pub fn new() -> Self {
        Self {
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

    pub fn push_node(&mut self, node: SyntaxNode) -> SyntaxId {
        let index = self.nodes.len() as u32;

        self.nodes.push(node);

        SyntaxId(index)
    }

    pub fn top_node(&self) -> Option<&SyntaxNode> {
        self.nodes.first()
    }

    pub fn get_node(&self, id: SyntaxId) -> Option<&SyntaxNode> {
        self.nodes.get(id.0 as usize)
    }

    pub fn get_children(&self, start_index: u32, count: u32) -> Option<&[SyntaxId]> {
        let start_index = start_index as usize;
        let count = count as usize;

        self.children.get(start_index..start_index + count)
    }

    pub fn add_children(&mut self, children: &[SyntaxId]) -> (u32, u32) {
        let start_index = self.children.len() as u32;
        let count = children.len() as u32;

        self.children.extend_from_slice(children);

        (start_index, count)
    }

    pub fn last_node(&self) -> Option<&SyntaxNode> {
        self.nodes.last()
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
                SyntaxNodeChildren::Double(left, right) => {
                    build_tree(&mut leaf, left, syntax_tree);
                    build_tree(&mut leaf, right, syntax_tree);
                }
                SyntaxNodeChildren::Multiple(start, count) => {
                    for child_id in syntax_tree.get_children(start, count).unwrap_or(&[]) {
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

        let top_node = match self.top_node() {
            Some(node) => node,
            None => return "<empty>".to_string(),
        };
        let mut root = Tree::new(*top_node);

        build_tree(&mut root, SyntaxId(0), self);
        root.to_string()
    }
}

impl Default for SyntaxTree {
    fn default() -> Self {
        Self::new()
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
