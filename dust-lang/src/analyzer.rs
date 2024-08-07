/// Tools for analyzing an abstract syntax tree and catch errors before running the virtual
/// machine.
///
/// This module provides to anlysis options, both of which borrow an abstract syntax tree and a
/// hash map of variables:
/// - `analyze` convenience function
/// - `Analyzer` struct
use std::collections::HashMap;

use crate::{AbstractSyntaxTree, Identifier, Node, Statement, Type, Value};

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
pub fn analyze<P: Clone>(
    abstract_tree: &AbstractSyntaxTree<P>,
    variables: &HashMap<Identifier, Value>,
) -> Result<(), AnalyzerError<P>> {
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
pub struct Analyzer<'a, P> {
    abstract_tree: &'a AbstractSyntaxTree<P>,
    variables: &'a HashMap<Identifier, Value>,
}

impl<'a, P: Clone> Analyzer<'a, P> {
    pub fn new(
        abstract_tree: &'a AbstractSyntaxTree<P>,
        variables: &'a HashMap<Identifier, Value>,
    ) -> Self {
        Self {
            abstract_tree,
            variables,
        }
    }

    pub fn analyze(&self) -> Result<(), AnalyzerError<P>> {
        for node in &self.abstract_tree.nodes {
            self.analyze_node(node)?;
        }

        Ok(())
    }

    fn analyze_node(&self, node: &Node<P>) -> Result<(), AnalyzerError<P>> {
        match &node.statement {
            Statement::Add(left, right) => {
                if let Some(Type::Integer) | Some(Type::Float) =
                    left.statement.expected_type(self.variables)
                {
                } else {
                    return Err(AnalyzerError::ExpectedIntegerOrFloat {
                        actual: left.as_ref().clone(),
                    });
                }

                if let Some(Type::Integer) | Some(Type::Float) =
                    right.statement.expected_type(self.variables)
                {
                } else {
                    return Err(AnalyzerError::ExpectedIntegerOrFloat {
                        actual: right.as_ref().clone(),
                    });
                }

                self.analyze_node(left)?;
                self.analyze_node(right)?;
            }
            Statement::Assign(left, right) => {
                if let Statement::Identifier(_) = &left.statement {
                    // Identifier is in the correct position
                } else {
                    return Err(AnalyzerError::ExpectedIdentifier {
                        actual: left.as_ref().clone(),
                    });
                }

                self.analyze_node(right)?;
            }
            Statement::BuiltInValue(node) => {
                self.analyze_node(node)?;
            }
            Statement::Constant(_) => {}
            Statement::Identifier(_) => {
                return Err(AnalyzerError::UnexpectedIdentifier {
                    identifier: node.clone(),
                });
            }
            Statement::List(statements) => {
                for statement in statements {
                    self.analyze_node(statement)?;
                }
            }
            Statement::Multiply(left, right) => {
                if let Some(Type::Integer) | Some(Type::Float) =
                    left.statement.expected_type(self.variables)
                {
                } else {
                    return Err(AnalyzerError::ExpectedIntegerOrFloat {
                        actual: left.as_ref().clone(),
                    });
                }

                if let Some(Type::Integer) | Some(Type::Float) =
                    right.statement.expected_type(self.variables)
                {
                } else {
                    return Err(AnalyzerError::ExpectedIntegerOrFloat {
                        actual: right.as_ref().clone(),
                    });
                }

                self.analyze_node(left)?;
                self.analyze_node(right)?;
            }
            Statement::PropertyAccess(left, right) => {
                if let Statement::Identifier(_) | Statement::Constant(_) | Statement::List(_) =
                    &left.statement
                {
                    // Left side is valid
                } else {
                    return Err(AnalyzerError::ExpectedIdentifier {
                        actual: left.as_ref().clone(),
                    });
                }

                self.analyze_node(right)?;
            }
            Statement::ReservedIdentifier(_) => {}
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnalyzerError<P> {
    ExpectedIdentifier { actual: Node<P> },
    ExpectedIntegerOrFloat { actual: Node<P> },
    UnexpectedIdentifier { identifier: Node<P> },
}

#[cfg(test)]
mod tests {
    use crate::{Identifier, Value};

    use super::*;

    #[test]
    fn multiply_expect_integer_or_float() {
        let abstract_tree = AbstractSyntaxTree {
            nodes: [Node::new(
                Statement::Multiply(
                    Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                    Box::new(Node::new(
                        Statement::Constant(Value::boolean(false)),
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
                actual: Node::new(Statement::Constant(Value::boolean(false)), (1, 2))
            })
        )
    }

    #[test]
    fn add_expect_integer_or_float() {
        let abstract_tree = AbstractSyntaxTree {
            nodes: [Node::new(
                Statement::Add(
                    Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                    Box::new(Node::new(
                        Statement::Constant(Value::boolean(false)),
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
                actual: Node::new(Statement::Constant(Value::boolean(false)), (1, 2))
            })
        )
    }

    #[test]
    fn assignment_expect_identifier() {
        let abstract_tree = AbstractSyntaxTree {
            nodes: [Node::new(
                Statement::Assign(
                    Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                    Box::new(Node::new(Statement::Constant(Value::integer(2)), (1, 2))),
                ),
                (0, 2),
            )]
            .into(),
        };
        let variables = HashMap::new();
        let analyzer = Analyzer::new(&abstract_tree, &variables);

        assert_eq!(
            analyzer.analyze(),
            Err(AnalyzerError::ExpectedIdentifier {
                actual: Node::new(Statement::Constant(Value::integer(1)), (0, 1))
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
                identifier: Node::new(Statement::Identifier(Identifier::new("x")), (0, 1))
            })
        )
    }
}
