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
    pub nodes: VecDeque<Node>,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Node {
    pub statement: Statement,
    pub position: Span,
}

impl Node {
    pub fn new(operation: Statement, position: Span) -> Self {
        Self {
            statement: operation,
            position,
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
    BuiltInFunctionCall {
        function: BuiltInFunction,
        type_arguments: Option<Vec<Node>>,
        value_arguments: Option<Vec<Node>>,
    },
    FunctionCall {
        function: Box<Node>,
        type_arguments: Option<Vec<Node>>,
        value_arguments: Option<Vec<Node>>,
    },
    PropertyAccess(Box<Node>, Box<Node>),
    List(Vec<Node>),
    Multiply(Box<Node>, Box<Node>),

    // Hard-coded values
    Constant(Value),
    Identifier(Identifier),
}

impl Statement {
    pub fn expected_type(&self, variables: &HashMap<Identifier, Value>) -> Option<Type> {
        match self {
            Statement::Add(left, _) => left.statement.expected_type(variables),
            Statement::Assign(_, _) => None,
            Statement::BuiltInFunctionCall { function, .. } => function.expected_type(),
            Statement::Constant(value) => Some(value.r#type(variables)),
            Statement::FunctionCall { function, .. } => function.statement.expected_type(variables),
            Statement::Identifier(identifier) => variables
                .get(identifier)
                .map(|value| value.r#type(variables)),
            Statement::List(_) => None,
            Statement::Multiply(left, _) => left.statement.expected_type(variables),
            Statement::PropertyAccess(_, _) => None,
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Assign(left, right) => write!(f, "{left} = {right}"),
            Statement::Add(left, right) => write!(f, "{left} + {right}"),
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
        }
    }
}
