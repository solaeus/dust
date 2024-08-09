//! Tools for analyzing an abstract syntax tree and catch errors before running the virtual
//! machine.
//!
//! This module provides to anlysis options, both of which borrow an abstract syntax tree and a
//! hash map of variables:
//! - `analyze` convenience function
//! - `Analyzer` struct
use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Display, Formatter},
};

use crate::{
    abstract_tree::BinaryOperator, AbstractSyntaxTree, BuiltInFunction, Identifier, Node, Span,
    Statement, Type, Value,
};

/// Analyzes the abstract syntax tree for errors.
///
/// # Examples
/// ```
/// # use std::collections::HashMap;
/// # use dust_lang::*;
/// let input = "x = 1 + false";
/// let abstract_tree = parse(input).unwrap();
/// let variables = HashMap::new();
/// let result = analyze(&abstract_tree, &variables);
///
/// assert!(result.is_err());
/// ```
pub fn analyze(
    abstract_tree: &AbstractSyntaxTree,
    variables: &HashMap<Identifier, Value>,
) -> Result<(), AnalyzerError> {
    let analyzer = Analyzer::new(abstract_tree, variables);

    analyzer.analyze()
}

/// Static analyzer that checks for potential runtime errors.
///
/// # Examples
/// ```
/// # use std::collections::HashMap;
/// # use dust_lang::*;
/// let input = "x = 1 + false";
/// let abstract_tree = parse(input).unwrap();
/// let variables = HashMap::new();
/// let analyzer = Analyzer::new(&abstract_tree, &variables);
/// let result = analyzer.analyze();
///
/// assert!(result.is_err());
pub struct Analyzer<'a> {
    abstract_tree: &'a AbstractSyntaxTree,
    variables: &'a HashMap<Identifier, Value>,
}

impl<'a> Analyzer<'a> {
    pub fn new(
        abstract_tree: &'a AbstractSyntaxTree,
        variables: &'a HashMap<Identifier, Value>,
    ) -> Self {
        Self {
            abstract_tree,
            variables,
        }
    }

    pub fn analyze(&self) -> Result<(), AnalyzerError> {
        for node in &self.abstract_tree.nodes {
            self.analyze_node(node)?;
        }

        Ok(())
    }

    fn analyze_node(&self, node: &Node<Statement>) -> Result<(), AnalyzerError> {
        match &node.inner {
            Statement::Assignment {
                identifier,
                value_node: value,
            } => {
                if let None = value.inner.expected_type(self.variables) {
                    return Err(AnalyzerError::ExpectedValue {
                        actual: value.as_ref().clone(),
                        position: value.position,
                    });
                }
            }
            Statement::BinaryOperation {
                left,
                operator,
                right,
            } => {
                self.analyze_node(left)?;
                self.analyze_node(right)?;
            }
            Statement::BuiltInFunctionCall { .. } => {}
            Statement::Constant(_) => {}
            Statement::FunctionCall { function, .. } => {
                if let Statement::Identifier(_) = &function.inner {
                    // Function is in the correct position
                } else {
                    return Err(AnalyzerError::ExpectedIdentifier {
                        actual: function.as_ref().clone(),
                        position: function.position,
                    });
                }
            }
            Statement::Identifier(_) => {
                return Err(AnalyzerError::UnexpectedIdentifier {
                    identifier: node.clone(),
                    position: node.position,
                });
            }
            Statement::List(statements) => {
                for statement in statements {
                    self.analyze_node(statement)?;
                }
            }
            Statement::PropertyAccess(left, right) => {
                if let Statement::Identifier(_) | Statement::Constant(_) | Statement::List(_) =
                    &left.inner
                {
                    // Left side is valid
                } else {
                    return Err(AnalyzerError::ExpectedValue {
                        actual: left.as_ref().clone(),
                        position: left.position,
                    });
                }

                if let Statement::BuiltInFunctionCall { function, .. } = &right.inner {
                    if function == &BuiltInFunction::IsEven || function == &BuiltInFunction::IsOdd {
                        if let Some(Type::Integer) = left.inner.expected_type(self.variables) {
                        } else {
                            return Err(AnalyzerError::ExpectedIntegerOrFloat {
                                actual: left.as_ref().clone(),
                                position: left.position,
                            });
                        }
                    }
                }

                self.analyze_node(right)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnalyzerError {
    ExpectedBoolean {
        actual: Node<Statement>,
        position: Span,
    },
    ExpectedFloat {
        actual: Node<Statement>,
        position: (usize, usize),
    },
    ExpectedFunction {
        actual: Node<Statement>,
        position: Span,
    },
    ExpectedIdentifier {
        actual: Node<Statement>,
        position: Span,
    },
    ExpectedInteger {
        actual: Node<Statement>,
        position: Span,
    },
    ExpectedIntegerOrFloat {
        actual: Node<Statement>,
        position: Span,
    },
    ExpectedIntegerFloatOrString {
        actual: Node<Statement>,
        position: Span,
    },
    ExpectedString {
        actual: Node<Statement>,
        position: (usize, usize),
    },
    ExpectedValue {
        actual: Node<Statement>,
        position: Span,
    },
    UnexpectedIdentifier {
        identifier: Node<Statement>,
        position: Span,
    },
    UnexectedString {
        actual: Node<Statement>,
        position: (usize, usize),
    },
}

impl AnalyzerError {
    pub fn position(&self) -> Span {
        match self {
            AnalyzerError::ExpectedBoolean { position, .. } => *position,
            AnalyzerError::ExpectedFloat { position, .. } => *position,
            AnalyzerError::ExpectedFunction { position, .. } => *position,
            AnalyzerError::ExpectedIdentifier { position, .. } => *position,
            AnalyzerError::ExpectedValue { position, .. } => *position,
            AnalyzerError::ExpectedInteger { position, .. } => *position,
            AnalyzerError::ExpectedIntegerOrFloat { position, .. } => *position,
            AnalyzerError::ExpectedIntegerFloatOrString { position, .. } => *position,
            AnalyzerError::ExpectedString { position, .. } => *position,
            AnalyzerError::UnexpectedIdentifier { position, .. } => *position,
            AnalyzerError::UnexectedString { position, .. } => *position,
        }
    }
}

impl Error for AnalyzerError {}

impl Display for AnalyzerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            AnalyzerError::ExpectedBoolean { actual, .. } => {
                write!(f, "Expected boolean, found {}", actual)
            }
            AnalyzerError::ExpectedFunction { actual, .. } => {
                write!(f, "Expected function, found {}", actual)
            }
            AnalyzerError::ExpectedFloat { actual, .. } => {
                write!(f, "Expected float, found {}", actual)
            }
            AnalyzerError::ExpectedIdentifier { actual, .. } => {
                write!(f, "Expected identifier, found {}", actual)
            }
            AnalyzerError::ExpectedInteger { actual, .. } => {
                write!(f, "Expected integer, found {}", actual)
            }
            AnalyzerError::ExpectedIntegerOrFloat { actual, .. } => {
                write!(f, "Expected integer or float, found {}", actual)
            }
            AnalyzerError::ExpectedIntegerFloatOrString { actual, .. } => {
                write!(f, "Expected integer, float, or string, found {}", actual)
            }
            AnalyzerError::ExpectedString { actual, .. } => {
                write!(f, "Expected string, found {}", actual)
            }
            AnalyzerError::ExpectedValue { actual, .. } => {
                write!(f, "Expected value, found {}", actual)
            }
            AnalyzerError::UnexpectedIdentifier { identifier, .. } => {
                write!(f, "Unexpected identifier {}", identifier)
            }
            AnalyzerError::UnexectedString { actual, .. } => {
                write!(f, "Unexpected string {}", actual)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BuiltInFunction, Identifier, Value};

    use super::*;

    #[test]
    fn add_expects_same_types() {
        let abstract_tree = AbstractSyntaxTree {
            nodes: [Node::new(
                Statement::BinaryOperation {
                    left: Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                    operator: Node::new(BinaryOperator::Add, (1, 2)),
                    right: Box::new(Node::new(Statement::Constant(Value::float(1.0)), (3, 4))),
                },
                (0, 4),
            )]
            .into(),
        };
        let variables = HashMap::new();
        let analyzer = Analyzer::new(&abstract_tree, &variables);

        assert_eq!(
            analyzer.analyze(),
            Err(AnalyzerError::ExpectedIntegerFloatOrString {
                actual: Node::new(Statement::Constant(Value::float(1.0)), (1, 2)),
                position: (1, 2)
            })
        )
    }

    #[test]
    fn add_expects_integer_float_or_string() {
        let abstract_tree = AbstractSyntaxTree {
            nodes: [Node::new(
                Statement::BinaryOperation {
                    left: Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                    operator: Node::new(BinaryOperator::Add, (1, 2)),
                    right: Box::new(Node::new(Statement::Constant(Value::boolean(true)), (3, 4))),
                },
                (0, 2),
            )]
            .into(),
        };
        let variables = HashMap::new();
        let analyzer = Analyzer::new(&abstract_tree, &variables);

        assert_eq!(
            analyzer.analyze(),
            Err(AnalyzerError::ExpectedIntegerFloatOrString {
                actual: Node::new(Statement::Constant(Value::boolean(true)), (0, 1)),
                position: (0, 1)
            })
        )
    }

    #[test]
    fn is_even_expects_number() {
        let abstract_tree = AbstractSyntaxTree {
            nodes: [Node::new(
                Statement::PropertyAccess(
                    Box::new(Node::new(Statement::Constant(Value::boolean(true)), (0, 1))),
                    Box::new(Node::new(
                        Statement::BuiltInFunctionCall {
                            function: BuiltInFunction::IsEven,
                            type_arguments: None,
                            value_arguments: None,
                        },
                        (1, 2),
                    )),
                ),
                (0, 2),
            )]
            .into(),
        };
        let variables = HashMap::new();
        let analyzer = Analyzer::new(&abstract_tree, &variables);

        assert_eq!(
            analyzer.analyze(),
            Err(AnalyzerError::ExpectedIntegerOrFloat {
                actual: Node::new(Statement::Constant(Value::boolean(true)), (0, 1)),
                position: (0, 1)
            })
        )
    }
    #[test]
    fn is_odd_expects_number() {
        let abstract_tree = AbstractSyntaxTree {
            nodes: [Node::new(
                Statement::PropertyAccess(
                    Box::new(Node::new(Statement::Constant(Value::boolean(true)), (0, 1))),
                    Box::new(Node::new(
                        Statement::BuiltInFunctionCall {
                            function: BuiltInFunction::IsOdd,
                            type_arguments: None,
                            value_arguments: None,
                        },
                        (1, 2),
                    )),
                ),
                (0, 2),
            )]
            .into(),
        };
        let variables = HashMap::new();
        let analyzer = Analyzer::new(&abstract_tree, &variables);

        assert_eq!(
            analyzer.analyze(),
            Err(AnalyzerError::ExpectedIntegerOrFloat {
                actual: Node::new(Statement::Constant(Value::boolean(true)), (0, 1)),
                position: (0, 1)
            })
        )
    }

    #[test]
    fn multiply_expect_integer_or_float() {
        let abstract_tree = AbstractSyntaxTree {
            nodes: [Node::new(
                Statement::BinaryOperation {
                    left: Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                    operator: Node::new(BinaryOperator::Multiply, (1, 2)),
                    right: Box::new(Node::new(
                        Statement::Constant(Value::boolean(false)),
                        (3, 4),
                    )),
                },
                (0, 2),
            )]
            .into(),
        };
        let variables = HashMap::new();
        let analyzer = Analyzer::new(&abstract_tree, &variables);

        assert_eq!(
            analyzer.analyze(),
            Err(AnalyzerError::ExpectedIntegerOrFloat {
                actual: Node::new(Statement::Constant(Value::boolean(false)), (1, 2)),
                position: (1, 2)
            })
        )
    }

    #[test]
    fn assignment_expect_identifier() {
        let abstract_tree = AbstractSyntaxTree {
            nodes: [Node::new(
                Statement::Assignment {
                    identifier: Identifier::new("x"),
                    value_node: Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                },
                (0, 2),
            )]
            .into(),
        };
        let variables = HashMap::new();
        let analyzer = Analyzer::new(&abstract_tree, &variables);

        assert_eq!(
            analyzer.analyze(),
            Err(AnalyzerError::ExpectedIdentifier {
                actual: Node::new(Statement::Constant(Value::integer(1)), (0, 1)),
                position: (0, 1)
            })
        )
    }

    #[test]
    fn unexpected_identifier() {
        let abstract_tree = AbstractSyntaxTree {
            nodes: [Node::new(
                Statement::Identifier(Identifier::new("x")),
                (0, 1),
            )]
            .into(),
        };
        let variables = HashMap::new();
        let analyzer = Analyzer::new(&abstract_tree, &variables);

        assert_eq!(
            analyzer.analyze(),
            Err(AnalyzerError::UnexpectedIdentifier {
                identifier: Node::new(Statement::Identifier(Identifier::new("x")), (0, 1)),
                position: (0, 1)
            })
        )
    }
}
