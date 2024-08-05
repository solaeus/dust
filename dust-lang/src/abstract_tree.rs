use crate::{Identifier, Span, Value};

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    pub operation: Statement,
    pub span: Span,
}

impl Node {
    pub fn new(operation: Statement, span: Span) -> Self {
        Self { operation, span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    // Top-level statements
    Assign(Box<(Node, Node)>),

    // Expressions
    Add(Box<(Node, Node)>),
    List(Vec<Node>),
    Multiply(Box<(Node, Node)>),

    // Hard-coded values
    Constant(Value),
    Identifier(Identifier),
}
