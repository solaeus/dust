use std::{
    collections::{HashMap, VecDeque},
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{Identifier, ReservedIdentifier, Span, Type, Value};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AbstractSyntaxTree {
    pub nodes: VecDeque<Node>,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
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

impl Display for Node {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.statement)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Statement {
    // Top-level statements
    Assign(Box<Node>, Box<Node>),

    // Expressions
    Add(Box<Node>, Box<Node>),
    BuiltInValue(Box<Node>),
    PropertyAccess(Box<Node>, Box<Node>),
    List(Vec<Node>),
    Multiply(Box<Node>, Box<Node>),

    // Hard-coded values
    Constant(Value),
    Identifier(Identifier),
    ReservedIdentifier(ReservedIdentifier),
}

impl Statement {
    pub fn expected_type(&self, variables: &HashMap<Identifier, Value>) -> Option<Type> {
        match self {
            Statement::Add(left, _) => left.statement.expected_type(variables),
            Statement::Assign(_, _) => None,
            Statement::BuiltInValue(reserved) => reserved.statement.expected_type(variables),
            Statement::Constant(value) => Some(value.r#type(variables)),
            Statement::Identifier(identifier) => variables
                .get(identifier)
                .map(|value| value.r#type(variables)),
            Statement::List(_) => None,
            Statement::Multiply(left, _) => left.statement.expected_type(variables),
            Statement::PropertyAccess(_, _) => None,
            Statement::ReservedIdentifier(reserved) => match reserved {
                ReservedIdentifier::IsEven | ReservedIdentifier::IsOdd => Some(Type::Boolean),
                ReservedIdentifier::Length => Some(Type::Integer),
            },
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Assign(left, right) => write!(f, "{left} = {right}"),
            Statement::Add(left, right) => write!(f, "{left} + {right}"),
            Statement::BuiltInValue(reserved) => write!(f, "{reserved}"),
            Statement::PropertyAccess(left, right) => write!(f, "{left}.{right}"),
            Statement::List(nodes) => {
                write!(f, "[")?;
                for (i, node) in nodes.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{node}")?;
                }
                write!(f, "]")
            }
            Statement::Multiply(left, right) => write!(f, "{left} * {right}"),
            Statement::Constant(value) => write!(f, "{value}"),
            Statement::Identifier(identifier) => write!(f, "{identifier}"),
            Statement::ReservedIdentifier(identifier) => write!(f, "{identifier}"),
        }
    }
}
