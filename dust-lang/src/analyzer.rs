//! Tools for analyzing an abstract syntax tree and catch errors before running the virtual
//! machine.
//!
//! This module provides to anlysis options, both of which borrow an abstract syntax tree and a
//! hash map of variables:
//! - `analyze` convenience function
//! - `Analyzer` struct
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use crate::{
    abstract_tree::BinaryOperator, AbstractSyntaxTree, BuiltInFunction, Context, Node, Span,
    Statement, Type,
};

/// Analyzes the abstract syntax tree for errors.
///
/// # Examples
/// ```
/// # use std::collections::HashMap;
/// # use dust_lang::*;
/// let input = "x = 1 + false";
/// let abstract_tree = parse(input).unwrap();
/// let mut context = Context::new();
/// let result = analyze(&abstract_tree, &mut context);
///
/// assert!(result.is_err());
/// ```
pub fn analyze(
    abstract_tree: &AbstractSyntaxTree,
    context: &mut Context,
) -> Result<(), AnalyzerError> {
    let mut analyzer = Analyzer::new(abstract_tree, context);

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
/// let mut context = Context::new();
/// let mut analyzer = Analyzer::new(&abstract_tree, &mut context);
/// let result = analyzer.analyze();
///
/// assert!(result.is_err());
pub struct Analyzer<'a> {
    abstract_tree: &'a AbstractSyntaxTree,
    context: &'a mut Context,
}

impl<'a> Analyzer<'a> {
    pub fn new(abstract_tree: &'a AbstractSyntaxTree, context: &'a mut Context) -> Self {
        Self {
            abstract_tree,
            context,
        }
    }

    pub fn analyze(&mut self) -> Result<(), AnalyzerError> {
        for node in &self.abstract_tree.nodes {
            self.analyze_node(node)?;
        }

        Ok(())
    }

    fn analyze_node(&mut self, node: &Node<Statement>) -> Result<(), AnalyzerError> {
        match &node.inner {
            Statement::BinaryOperation {
                left,
                operator,
                right,
            } => {
                if let BinaryOperator::Assign = operator.inner {
                    if let Statement::Identifier(identifier) = &left.inner {
                        self.analyze_node(right)?;

                        let right_type = right.inner.expected_type(self.context);

                        self.context.set_type(
                            identifier.clone(),
                            right_type.ok_or(AnalyzerError::ExpectedValue {
                                actual: right.as_ref().clone(),
                                position: right.position,
                            })?,
                        );

                        return Ok(());
                    }
                }

                self.analyze_node(left)?;
                self.analyze_node(right)?;

                let left_type = left.inner.expected_type(self.context);
                let right_type = right.inner.expected_type(self.context);

                if let BinaryOperator::Add
                | BinaryOperator::Subtract
                | BinaryOperator::Multiply
                | BinaryOperator::Divide
                | BinaryOperator::Greater
                | BinaryOperator::GreaterOrEqual
                | BinaryOperator::Less
                | BinaryOperator::LessOrEqual = operator.inner
                {
                    match (left_type, right_type) {
                        (Some(Type::Integer), Some(Type::Integer)) => {}
                        (Some(Type::Float), Some(Type::Float)) => {}
                        (Some(Type::String), Some(Type::String)) => {}
                        (Some(Type::Integer), _) => {
                            return Err(AnalyzerError::ExpectedInteger {
                                actual: right.as_ref().clone(),
                                position: right.position,
                            });
                        }
                        (Some(Type::Float), _) => {
                            return Err(AnalyzerError::ExpectedFloat {
                                actual: right.as_ref().clone(),
                                position: right.position,
                            });
                        }
                        (Some(Type::String), _) => {
                            return Err(AnalyzerError::ExpectedString {
                                actual: right.as_ref().clone(),
                                position: right.position,
                            });
                        }
                        (_, _) => {
                            return Err(AnalyzerError::ExpectedIntegerFloatOrString {
                                actual: right.as_ref().clone(),
                                position: right.position,
                            })
                        }
                    }
                }
            }
            Statement::Block(statements) => {
                for statement in statements {
                    self.analyze_node(statement)?;
                }
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
            Statement::Identifier(identifier) => {
                let exists = self.context.add_allowed_use(identifier);

                if !exists {
                    return Err(AnalyzerError::UndefinedVariable {
                        identifier: node.clone(),
                    });
                }
            }
            Statement::List(statements) => {
                for statement in statements {
                    self.analyze_node(statement)?;
                }
            }
            Statement::Map(properties) => {
                for (_key, value_node) in properties {
                    self.analyze_node(value_node)?;
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
                        if let Some(Type::Integer) = left.inner.expected_type(self.context) {
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
            Statement::Nil(node) => {
                self.analyze_node(node)?;
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
    UndefinedVariable {
        identifier: Node<Statement>,
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
            AnalyzerError::UndefinedVariable { identifier } => identifier.position,
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
            AnalyzerError::UndefinedVariable { identifier } => {
                write!(f, "Undefined variable {}", identifier)
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
    fn math_requires_same_types() {
        let abstract_tree = AbstractSyntaxTree {
            nodes: [Node::new(
                Statement::BinaryOperation {
                    left: Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                    operator: Node::new(BinaryOperator::Add, (1, 2)),
                    right: Box::new(Node::new(Statement::Constant(Value::float(1.0)), (3, 4))),
                },
                (0, 2),
            )]
            .into(),
        };
        let mut context = Context::new();
        let mut analyzer = Analyzer::new(&abstract_tree, &mut context);

        assert_eq!(
            analyzer.analyze(),
            Err(AnalyzerError::ExpectedInteger {
                actual: Node::new(Statement::Constant(Value::float(1.0)), (3, 4)),
                position: (3, 4)
            })
        )
    }

    #[test]
    fn float_plus_integer() {
        let abstract_tree = AbstractSyntaxTree {
            nodes: [Node::new(
                Statement::BinaryOperation {
                    left: Box::new(Node::new(Statement::Constant(Value::float(1.0)), (0, 1))),
                    operator: Node::new(BinaryOperator::Add, (1, 2)),
                    right: Box::new(Node::new(Statement::Constant(Value::integer(1)), (3, 4))),
                },
                (0, 2),
            )]
            .into(),
        };
        let mut context = Context::new();
        let mut analyzer = Analyzer::new(&abstract_tree, &mut context);

        assert_eq!(
            analyzer.analyze(),
            Err(AnalyzerError::ExpectedFloat {
                actual: Node::new(Statement::Constant(Value::integer(1)), (3, 4)),
                position: (3, 4)
            })
        )
    }

    #[test]
    fn integer_plus_boolean() {
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
        let mut context = Context::new();
        let mut analyzer = Analyzer::new(&abstract_tree, &mut context);

        assert_eq!(
            analyzer.analyze(),
            Err(AnalyzerError::ExpectedInteger {
                actual: Node::new(Statement::Constant(Value::boolean(true)), (3, 4)),
                position: (3, 4)
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
        let mut context = Context::new();
        let mut analyzer = Analyzer::new(&abstract_tree, &mut context);

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
        let mut context = Context::new();
        let mut analyzer = Analyzer::new(&abstract_tree, &mut context);

        assert_eq!(
            analyzer.analyze(),
            Err(AnalyzerError::ExpectedIntegerOrFloat {
                actual: Node::new(Statement::Constant(Value::boolean(true)), (0, 1)),
                position: (0, 1)
            })
        )
    }

    #[test]
    fn undefined_variable() {
        let abstract_tree = AbstractSyntaxTree {
            nodes: [Node::new(
                Statement::Identifier(Identifier::new("x")),
                (0, 1),
            )]
            .into(),
        };
        let mut context = Context::new();
        let mut analyzer = Analyzer::new(&abstract_tree, &mut context);

        assert_eq!(
            analyzer.analyze(),
            Err(AnalyzerError::UndefinedVariable {
                identifier: Node::new(Statement::Identifier(Identifier::new("x")), (0, 1))
            })
        )
    }
}
