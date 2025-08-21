mod local;
mod scope;
mod syntax_node;

pub use local::Local;
pub use scope::Scope;
pub use syntax_node::{SyntaxKind, SyntaxNode};

use serde::{Deserialize, Serialize};

use crate::{Type, Value};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SyntaxTree {
    /// Sequence of nodes that represent the source code. Their order is according to how they are
    /// parsed according to the Pratt algorithm.
    pub nodes: Vec<SyntaxNode>,

    /// Concatenated list of node indexes that represent children for nodes whose child indexes
    /// cannot be stored directly in the node (i.e. blocks and the root node).
    pub children: Vec<u32>,

    /// Hard-coded values that are used in the source code.
    pub constants: Vec<Value>,

    /// Limitted-scope variables that are defined in the source code.
    pub locals: Vec<Local>,
}

impl SyntaxTree {
    pub fn node_count(&self) -> u32 {
        self.nodes.len() as u32
    }

    pub fn push_node(&mut self, node: SyntaxNode) {
        self.nodes.push(node);
    }

    pub fn get_node(&self, index: u32) -> Option<&SyntaxNode> {
        self.nodes.get(index as usize)
    }

    pub fn last_node(&self) -> Option<&SyntaxNode> {
        self.nodes.last()
    }

    pub fn push_constant(&mut self, value: Value) -> u32 {
        if let Some(index) = self
            .constants
            .iter()
            .position(|constant| constant == &value)
        {
            return index as u32;
        }

        let index = self.constants.len() as u32;

        self.constants.push(value);

        index
    }

    pub fn push_local(&mut self, local: Local) -> Result<u32, u32> {
        if let Some(index) = self
            .locals
            .iter()
            .position(|existing| existing.identifier_position == local.identifier_position)
        {
            return Err(index as u32);
        }

        let index = self.locals.len() as u32;

        self.locals.push(local);

        Ok(index)
    }

    pub fn get_local(&self, index: u32) -> Option<&Local> {
        self.locals.get(index as usize)
    }

    pub fn find_local(&self, identifier: &str, source: &str) -> Option<&Local> {
        self.locals.iter().find(|local| {
            let local_identifier = &source[local.identifier_position.as_usize_range()];

            local_identifier == identifier
        })
    }

    pub fn find_local_index(&self, identifier: &str, source: &str) -> Option<u32> {
        self.locals
            .iter()
            .position(|local| {
                let local_identifier = &source[local.identifier_position.as_usize_range()];

                local_identifier == identifier
            })
            .map(|index| index as u32)
    }

    pub fn resolve_type(&self, node_index: u32) -> Type {
        let Some(node) = self.get_node(node_index) else {
            return Type::None;
        };

        match node.kind {
            SyntaxKind::BooleanExpression => Type::Boolean,
            SyntaxKind::ByteExpression => Type::Byte,
            SyntaxKind::FloatExpression => Type::Float,
            SyntaxKind::IntegerExpression => Type::Integer,
            SyntaxKind::StringExpression => Type::String,
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression => self.resolve_type(node.child),
            SyntaxKind::GroupedExpression => self.resolve_type(node.child),
            SyntaxKind::LetStatement => Type::None,
            _ => todo!(),
        }
    }

    pub fn display_node_tree(&self) -> String {
        let mut output = String::new();
        let main_node = self.nodes[0];

        output.push_str("Syntax Tree:\n");
        self.display_node(main_node, 0, &mut output);

        output
    }

    pub fn display_node(&self, node: SyntaxNode, depth: usize, output: &mut String) {
        let indent = "  ".repeat(depth);
        let node_display = if depth == 0 {
            format!("{}", node.kind)
        } else {
            format!("{}- {}", indent, node.kind)
        };

        output.push('\n');
        output.push_str(&node_display);

        match node.kind {
            SyntaxKind::MainFunctionStatement => {
                let children_start = node.child as usize;
                let children_end = children_start + node.payload as usize;
                let children = &self.children[children_start..children_end];

                for &child_index in children {
                    let child = self.nodes[child_index as usize];

                    self.display_node(child, 1, output);
                }
            }
            SyntaxKind::ExpressionStatement => {
                let expression = self.nodes[node.child as usize];

                self.display_node(expression, depth + 1, output);
            }
            SyntaxKind::LetStatement => {
                let expression = self.nodes[node.child as usize];

                self.display_node(expression, depth + 1, output);
            }
            SyntaxKind::IntegerExpression => {
                let constant_index = node.payload as usize;
                let constant = &self.constants[constant_index];
                let integer_display = format!(": {constant}");

                output.push_str(&integer_display);
            }
            SyntaxKind::AdditionExpression => {
                let left = self.nodes[node.child as usize];
                let right = self.nodes[node.payload as usize];

                self.display_node(left, depth + 1, output);
                self.display_node(right, depth + 1, output);
            }
            SyntaxKind::SubtractionExpression => {
                let left = self.nodes[node.child as usize];
                let right = self.nodes[node.payload as usize];

                self.display_node(left, depth + 1, output);
                self.display_node(right, depth + 1, output);
            }
            SyntaxKind::MultiplicationExpression => {
                let left = self.nodes[node.child as usize];
                let right = self.nodes[node.payload as usize];

                self.display_node(left, depth + 1, output);
                self.display_node(right, depth + 1, output);
            }
            SyntaxKind::DivisionExpression => {
                let left = self.nodes[node.child as usize];
                let right = self.nodes[node.payload as usize];

                self.display_node(left, depth + 1, output);
                self.display_node(right, depth + 1, output);
            }
            SyntaxKind::ModuloExpression => {
                let left = self.nodes[node.child as usize];
                let right = self.nodes[node.payload as usize];

                self.display_node(left, depth + 1, output);
                self.display_node(right, depth + 1, output);
            }
            SyntaxKind::GroupedExpression => {
                let expression = self.nodes[node.child as usize];

                self.display_node(expression, depth + 1, output);
            }
            _ => {
                output.push_str(&indent);
                output.push_str("<todo>")
            }
        }
    }
}
