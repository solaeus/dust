//! In-memory representation of a Dust program.
use std::{
    collections::{BTreeMap, HashMap, VecDeque},
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
    // A sequence of statements
    Block(Vec<Node<Statement>>),

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
    Map(Vec<(Node<Statement>, Node<Statement>)>),

    // Hard-coded value
    Constant(Value),

    // A statement that always returns None. Created with a semicolon, it causes the preceding
    // statement to return None. This is analagous to the semicolon or unit type in Rust.
    Nil(Box<Node<Statement>>),
}

impl Statement {
    pub fn expected_type(&self, variables: &HashMap<Identifier, Value>) -> Option<Type> {
        match self {
            Statement::Block(nodes) => nodes.last().unwrap().inner.expected_type(variables),
            Statement::BinaryOperation { left, .. } => left.inner.expected_type(variables),
            Statement::BuiltInFunctionCall { function, .. } => function.expected_return_type(),
            Statement::Constant(value) => Some(value.r#type(variables)),
            Statement::FunctionCall { function, .. } => function.inner.expected_type(variables),
            Statement::Identifier(identifier) => variables
                .get(identifier)
                .map(|value| value.r#type(variables)),
            Statement::List(nodes) => {
                let item_type = nodes.first().unwrap().inner.expected_type(variables)?;

                Some(Type::List {
                    length: nodes.len(),
                    item_type: Box::new(item_type),
                })
            }
            Statement::Map(nodes) => {
                let mut types = BTreeMap::new();

                for (identifier, item) in nodes {
                    if let Statement::Identifier(identifier) = &identifier.inner {
                        types.insert(identifier.clone(), item.inner.expected_type(variables)?);
                    }
                }

                Some(Type::Map(types))
            }
            Statement::PropertyAccess(_, _) => None,
            Statement::Nil(_) => None,
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Block(statements) => {
                write!(f, "{{ ")?;

                for (i, statement) in statements.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }

                    write!(f, "{statement}")?;
                }

                write!(f, " }}")
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
            Statement::Map(nodes) => {
                write!(f, "{{")?;

                for (i, (identifier, node)) in nodes.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{identifier} = {node}")?;
                }

                write!(f, "}}")
            }
            Statement::Nil(node) => write!(f, "{node};"),
            Statement::PropertyAccess(left, right) => write!(f, "{left}.{right}"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BinaryOperator {
    // Math
    Add,
    Divide,
    Modulo,
    Multiply,
    Subtract,

    // Comparison
    Equal,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,

    // Logic
    And,
    Or,

    // Assignment
    Assign,
    AddAssign,
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::AddAssign => write!(f, "+="),
            BinaryOperator::Assign => write!(f, "="),
            BinaryOperator::And => write!(f, "&&"),
            BinaryOperator::Divide => write!(f, "/"),
            BinaryOperator::Equal => write!(f, "=="),
            BinaryOperator::Greater => write!(f, ">"),
            BinaryOperator::GreaterOrEqual => write!(f, ">="),
            BinaryOperator::Less => write!(f, "<"),
            BinaryOperator::LessOrEqual => write!(f, "<="),
            BinaryOperator::Modulo => write!(f, "%"),
            BinaryOperator::Multiply => write!(f, "*"),
            BinaryOperator::Or => write!(f, "||"),
            BinaryOperator::Subtract => write!(f, "-"),
        }
    }
}
