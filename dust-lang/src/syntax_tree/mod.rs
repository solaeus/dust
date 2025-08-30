mod syntax_node;

use serde_json::de;
pub use syntax_node::{SyntaxKind, SyntaxNode};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SyntaxId(pub u32);

/// Lossless abstract syntax tree representing a Dust source code file.
#[derive(Debug, Default)]
pub struct SyntaxTree {
    /// List of nodes in the tree in the order they were parsed according to the Pratt algorithm
    /// used by the parser.
    pub nodes: Vec<SyntaxNode>,

    /// Concatenated list of node indexes that represent children for nodes whose child indexes
    /// cannot be stored directly in the node (i.e. blocks and the root node).
    pub children: Vec<SyntaxId>,
}

impl SyntaxTree {
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

    pub fn is_subtree(&self) -> bool {
        self.nodes.first().is_some_and(|node| {
            !matches!(
                node.kind,
                SyntaxKind::MainFunctionItem | SyntaxKind::ModuleItem
            )
        })
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

    pub fn last_node(&self) -> Option<&SyntaxNode> {
        self.nodes.last()
    }

    pub fn display(&self) -> String {
        let mut output = String::new();

        if let Some(top_node) = self.top_node() {
            output.push_str("Syntax Tree:\n");
            self.display_node(top_node, 0, &mut output);
        } else {
            output.push_str(" <empty>");
        }

        output
    }

    pub fn display_node(&self, node: &SyntaxNode, depth: usize, output: &mut String) {
        let push_error = |output: &mut String| {
            output.push_str("\n  <error>");
        };

        let indent = "  ".repeat(depth);
        let node_display = if depth == 0 {
            format!("{}", node.kind)
        } else {
            format!("{}- {}", indent, node.kind)
        };

        output.push('\n');
        output.push_str(&node_display);

        match node.kind {
            SyntaxKind::MainFunctionItem => {
                if depth != 0 {
                    output.push_str(" <error: main function must be root node>");

                    return;
                }

                let children_start = node.payload.0 as usize;
                let children_end = children_start + node.payload.1 as usize;
                let children = &self.children[children_start..children_end];

                for child_id in children {
                    if let Some(child) = self.get_node(*child_id) {
                        self.display_node(child, 1, output);
                    } else {
                        push_error(output);
                    }
                }
            }
            SyntaxKind::LetStatement
            | SyntaxKind::ExpressionStatement
            | SyntaxKind::GroupedExpression => {
                if let Some(expression) = self.nodes.get(node.payload.1 as usize) {
                    self.display_node(expression, depth + 1, output);
                } else {
                    push_error(output);
                }
            }
            SyntaxKind::BooleanExpression => {
                let boolean = node.payload.1 != 0;
                let boolean_display = format!(": {boolean}");

                output.push_str(&boolean_display);
            }
            SyntaxKind::ByteExpression => {
                let byte = node.payload.0 as u8;
                let byte_display = format!(": {byte:02x}");

                output.push_str(&byte_display);
            }
            SyntaxKind::CharacterExpression => {
                let character_display = char::from_u32(node.payload.0)
                    .map(|character| format!(": '{character}'"))
                    .unwrap_or_else(|| "<error: invalid character>".to_string());

                output.push_str(&character_display);
            }
            SyntaxKind::FloatExpression => {
                let mut bytes = [0u8; 8];

                bytes.copy_from_slice(&node.payload.0.to_le_bytes());
                bytes.copy_from_slice(&node.payload.1.to_le_bytes());

                let float_display = SyntaxNode::decode_float(node.payload).to_string();

                output.push_str(&float_display);
            }
            SyntaxKind::IntegerExpression => {
                let integer_value = node.payload.0 as i64;
                let integer_display = format!(": {integer_value}", integer_value = integer_value);

                output.push_str(&integer_display);
            }
            SyntaxKind::StringExpression => {
                let string_index = node.payload.0 as usize;
                let string_display = format!("<string constant: {string_index}>");

                output.push_str(&string_display);
            }
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression => {
                if let Some(left_expression) = self.nodes.get(node.payload.0 as usize) {
                    self.display_node(left_expression, depth + 1, output);
                } else {
                    push_error(output);
                }

                if let Some(right_expression) = self.nodes.get(node.payload.1 as usize) {
                    self.display_node(right_expression, depth + 1, output);
                } else {
                    push_error(output);
                }
            }
            _ => {}
        }
    }
}
