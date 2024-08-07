use std::{
    collections::{HashMap, VecDeque},
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{Identifier, ReservedIdentifier, Type, Value};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AbstractSyntaxTree<P> {
    pub nodes: VecDeque<Node<P>>,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Node<P> {
    pub statement: Statement<P>,
    pub position: P,
}

impl<P> Node<P> {
    pub fn new(operation: Statement<P>, position: P) -> Self {
        Self {
            statement: operation,
            position,
        }
    }
}

impl<P> Display for Node<P> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.statement)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Statement<P> {
    // Top-level statements
    Assign(Box<Node<P>>, Box<Node<P>>),

    // Expressions
    Add(Box<Node<P>>, Box<Node<P>>),
    BuiltInValue(Box<Node<P>>),
    PropertyAccess(Box<Node<P>>, Box<Node<P>>),
    List(Vec<Node<P>>),
    Multiply(Box<Node<P>>, Box<Node<P>>),

    // Hard-coded values
    Constant(Value),
    Identifier(Identifier),
    ReservedIdentifier(ReservedIdentifier),
}

impl<P> Statement<P> {
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

impl<P> Display for Statement<P> {
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
