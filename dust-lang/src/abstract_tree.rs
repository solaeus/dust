//! In-memory representation of a Dust program.
use std::{
    collections::{HashMap, VecDeque},
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{BuiltInFunction, Identifier, Span, Type, Value};

/// In-memory representation of a Dust program.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AbstractSyntaxTree {
    pub nodes: VecDeque<Node<Statement>>,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Node<T> {
    pub inner: T,
    pub position: Span,
}

impl<T> Node<T> {
    pub fn new(inner: T, position: Span) -> Self {
        Self { inner, position }
    }
}

impl<T: Display> Display for Node<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Statement {
    // Top-level statements
    Assignment {
        identifier: Identifier,
        value_node: Box<Node<Statement>>,
    },

    // Logic, math and comparison expressions
    BinaryOperation {
        left: Box<Node<Statement>>,
        operator: Node<BinaryOperator>,
        right: Box<Node<Statement>>,
    },

    // Function calls
    BuiltInFunctionCall {
        function: BuiltInFunction,
        type_arguments: Option<Vec<Node<Statement>>>,
        value_arguments: Option<Vec<Node<Statement>>>,
    },
    FunctionCall {
        function: Box<Node<Statement>>,
        type_arguments: Option<Vec<Node<Statement>>>,
        value_arguments: Option<Vec<Node<Statement>>>,
    },

    // Property access expression
    PropertyAccess(Box<Node<Statement>>, Box<Node<Statement>>),

    // Identifier expression
    Identifier(Identifier),

    // Value collection expressions
    List(Vec<Node<Statement>>),

    // Hard-coded values
    Constant(Value),
}

impl Statement {
    pub fn expected_type(&self, variables: &HashMap<Identifier, Value>) -> Option<Type> {
        match self {
            Statement::Assignment { .. } => None,
            Statement::BinaryOperation { left, .. } => left.inner.expected_type(variables),
            Statement::BuiltInFunctionCall { function, .. } => function.expected_return_type(),
            Statement::Constant(value) => Some(value.r#type(variables)),
            Statement::FunctionCall { function, .. } => function.inner.expected_type(variables),
            Statement::Identifier(identifier) => variables
                .get(identifier)
                .map(|value| value.r#type(variables)),
            Statement::List(nodes) => nodes
                .first()
                .map(|node| node.inner.expected_type(variables))
                .flatten(),
            Statement::PropertyAccess(_, _) => None,
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Assignment {
                identifier,
                value_node: value,
            } => {
                write!(f, "{identifier} = {value}")
            }
            Statement::BinaryOperation {
                left,
                operator,
                right,
            } => {
                write!(f, "{left} {operator} {right}")
            }
            Statement::BuiltInFunctionCall {
                function,
                type_arguments: type_parameters,
                value_arguments: value_parameters,
            } => {
                write!(f, "{function}")?;

                if let Some(type_parameters) = type_parameters {
                    write!(f, "<")?;

                    for (i, type_parameter) in type_parameters.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }

                        write!(f, "{type_parameter}")?;
                    }

                    write!(f, ">")?;
                }

                write!(f, "(")?;

                if let Some(value_parameters) = value_parameters {
                    for (i, value_parameter) in value_parameters.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }

                        write!(f, "{value_parameter}")?;
                    }
                }

                write!(f, ")")
            }
            Statement::Constant(value) => write!(f, "{value}"),
            Statement::FunctionCall {
                function,
                type_arguments: type_parameters,
                value_arguments: value_parameters,
            } => {
                write!(f, "{function}")?;

                if let Some(type_parameters) = type_parameters {
                    write!(f, "<")?;

                    for (i, type_parameter) in type_parameters.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }

                        write!(f, "{type_parameter}")?;
                    }

                    write!(f, ">")?;
                }

                write!(f, "(")?;

                if let Some(value_parameters) = value_parameters {
                    for (i, value_parameter) in value_parameters.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }

                        write!(f, "{value_parameter}")?;
                    }
                }

                write!(f, ")")
            }
            Statement::Identifier(identifier) => write!(f, "{identifier}"),
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
            Statement::PropertyAccess(left, right) => write!(f, "{left}.{right}"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,
    Divide,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
    Multiply,
    Subtract,
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Divide => write!(f, "/"),
            BinaryOperator::Greater => write!(f, ">"),
            BinaryOperator::GreaterOrEqual => write!(f, ">="),
            BinaryOperator::Less => write!(f, "<"),
            BinaryOperator::LessOrEqual => write!(f, "<="),
            BinaryOperator::Multiply => write!(f, "*"),
            BinaryOperator::Subtract => write!(f, "-"),
        }
    }
}
