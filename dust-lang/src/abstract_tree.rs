use crate::{Identifier, ReservedIdentifier, Span, Value};

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    pub statement: Statement,
    pub span: Span,
}

impl Node {
    pub fn new(operation: Statement, span: Span) -> Self {
        Self {
            statement: operation,
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    // Top-level statements
    Assign(Box<Node>, Box<Node>),

    // Expressions
    Add(Box<Node>, Box<Node>),
    PropertyAccess(Box<Node>, Box<Node>),
    List(Vec<Node>),
    Multiply(Box<Node>, Box<Node>),

    // Hard-coded values
    Constant(Value),
    Identifier(Identifier),
    ReservedIdentifier(ReservedIdentifier),
}
