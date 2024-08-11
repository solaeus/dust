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
    abstract_tree::BinaryOperator, parse, AbstractSyntaxTree, Context, DustError, Node, Span,
    Statement, Type,
};

/// Analyzes the abstract syntax tree for errors.
///
/// # Examples
/// ```
/// # use std::collections::HashMap;
/// # use dust_lang::*;
/// let input = "x = 1 + false";
/// let result = analyze(input);
///
/// assert!(result.is_err());
/// ```
pub fn analyze(source: &str) -> Result<(), DustError> {
    let abstract_tree = parse(source)?;
    let mut context = Context::new();
    let mut analyzer = Analyzer::new(&abstract_tree, &mut context);

    analyzer
        .analyze()
        .map_err(|analyzer_error| DustError::AnalyzerError {
            analyzer_error,
            source,
        })
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
                    if let Some(expected_type) = left_type {
                        if let Some(actual_type) = right_type {
                            expected_type.check(&actual_type).map_err(|conflict| {
                                AnalyzerError::TypeConflict {
                                    actual_statement: right.as_ref().clone(),
                                    actual_type: conflict.actual,
                                    expected: conflict.expected,
                                }
                            })?;
                        } else {
                            return Err(AnalyzerError::ExpectedValue {
                                actual: right.as_ref().clone(),
                            });
                        }
                    }
                }
            }
            Statement::Block(statements) => {
                for statement in statements {
                    self.analyze_node(statement)?;
                }
            }
            Statement::BuiltInFunctionCall {
                function,
                value_arguments,
                ..
            } => {
                let value_parameters = function.value_parameters();

                if let Some(arguments) = value_arguments {
                    for argument in arguments {
                        self.analyze_node(argument)?;
                    }

                    if arguments.len() != value_parameters.len() {
                        return Err(AnalyzerError::ExpectedValueArgumentCount {
                            expected: value_parameters.len(),
                            actual: arguments.len(),
                            position: node.position,
                        });
                    }

                    for ((_identifier, parameter_type), argument) in
                        value_parameters.iter().zip(arguments)
                    {
                        let argument_type_option = argument.inner.expected_type(self.context);

                        if let Some(argument_type) = argument_type_option {
                            parameter_type.check(&argument_type).map_err(|conflict| {
                                AnalyzerError::TypeConflict {
                                    actual_statement: argument.clone(),
                                    actual_type: conflict.actual,
                                    expected: parameter_type.clone(),
                                }
                            })?;
                        } else {
                            return Err(AnalyzerError::ExpectedValue {
                                actual: argument.clone(),
                            });
                        }
                    }

                    if arguments.is_empty() && !value_parameters.is_empty() {
                        return Err(AnalyzerError::ExpectedValueArgumentCount {
                            expected: value_parameters.len(),
                            actual: 0,
                            position: node.position,
                        });
                    }
                } else if !value_parameters.is_empty() {
                    return Err(AnalyzerError::ExpectedValueArgumentCount {
                        expected: value_parameters.len(),
                        actual: 0,
                        position: node.position,
                    });
                }
            }
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
            Statement::If { condition, body } => {
                self.analyze_node(condition)?;

                if let Some(Type::Boolean) = condition.inner.expected_type(self.context) {
                    // Condition is valid
                } else {
                    return Err(AnalyzerError::ExpectedBoolean {
                        actual: condition.as_ref().clone(),
                        position: condition.position,
                    });
                }

                self.analyze_node(body)?;
            }
            Statement::IfElse {
                condition,
                if_body,
                else_body,
            } => {
                self.analyze_node(condition)?;

                if let Some(Type::Boolean) = condition.inner.expected_type(self.context) {
                    // Condition is valid
                } else {
                    return Err(AnalyzerError::ExpectedBoolean {
                        actual: condition.as_ref().clone(),
                        position: condition.position,
                    });
                }

                self.analyze_node(if_body)?;
                self.analyze_node(else_body)?;
            }
            Statement::IfElseIf {
                condition,
                if_body,
                else_ifs,
            } => {
                self.analyze_node(condition)?;

                if let Some(Type::Boolean) = condition.inner.expected_type(self.context) {
                    // Condition is valid
                } else {
                    return Err(AnalyzerError::ExpectedBoolean {
                        actual: condition.as_ref().clone(),
                        position: condition.position,
                    });
                }

                self.analyze_node(if_body)?;

                for (condition, body) in else_ifs {
                    self.analyze_node(condition)?;

                    if let Some(Type::Boolean) = condition.inner.expected_type(self.context) {
                        // Condition is valid
                    } else {
                        return Err(AnalyzerError::ExpectedBoolean {
                            actual: condition.clone(),
                            position: condition.position,
                        });
                    }

                    self.analyze_node(body)?;
                }
            }
            Statement::IfElseIfElse {
                condition,
                if_body,
                else_ifs,
                else_body,
            } => {
                self.analyze_node(condition)?;

                if let Some(Type::Boolean) = condition.inner.expected_type(self.context) {
                    // Condition is valid
                } else {
                    return Err(AnalyzerError::ExpectedBoolean {
                        actual: condition.as_ref().clone(),
                        position: condition.position,
                    });
                }

                self.analyze_node(if_body)?;

                for (condition, body) in else_ifs {
                    self.analyze_node(condition)?;

                    if let Some(Type::Boolean) = condition.inner.expected_type(self.context) {
                        // Condition is valid
                    } else {
                        return Err(AnalyzerError::ExpectedBoolean {
                            actual: condition.clone(),
                            position: condition.position,
                        });
                    }

                    self.analyze_node(body)?;
                }

                self.analyze_node(else_body)?;
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
            Statement::Nil(node) => {
                self.analyze_node(node)?;
            }
            Statement::PropertyAccess(left, right) => {
                if let Statement::Identifier(_) | Statement::Constant(_) | Statement::List(_) =
                    &left.inner
                {
                    // Left side is valid
                } else {
                    return Err(AnalyzerError::ExpectedValue {
                        actual: left.as_ref().clone(),
                    });
                }

                self.analyze_node(left)?;
                self.analyze_node(right)?;
            }
            Statement::While { condition, body } => {
                self.analyze_node(condition)?;
                self.analyze_node(body)?;

                if let Some(Type::Boolean) = condition.inner.expected_type(self.context) {
                } else {
                    return Err(AnalyzerError::ExpectedBoolean {
                        actual: condition.as_ref().clone(),
                        position: condition.position,
                    });
                }
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
    ExpectedNumber {
        actual: Node<Statement>,
        position: Span,
    },
    ExpectedNumberOrString {
        actual: Node<Statement>,
        position: Span,
    },
    ExpectedString {
        actual: Node<Statement>,
        position: (usize, usize),
    },
    ExpectedValue {
        actual: Node<Statement>,
    },
    ExpectedValueArgumentCount {
        expected: usize,
        actual: usize,
        position: Span,
    },
    TypeConflict {
        actual_statement: Node<Statement>,
        actual_type: Type,
        expected: Type,
    },
    UndefinedVariable {
        identifier: Node<Statement>,
    },
    UnexpectedIdentifier {
        identifier: Node<Statement>,
    },
    UnexectedString {
        actual: Node<Statement>,
    },
}

impl AnalyzerError {
    pub fn position(&self) -> Span {
        match self {
            AnalyzerError::ExpectedBoolean { position, .. } => *position,
            AnalyzerError::ExpectedFloat { position, .. } => *position,
            AnalyzerError::ExpectedFunction { position, .. } => *position,
            AnalyzerError::ExpectedIdentifier { position, .. } => *position,
            AnalyzerError::ExpectedValue { actual } => actual.position,
            AnalyzerError::ExpectedInteger { position, .. } => *position,
            AnalyzerError::ExpectedNumber { position, .. } => *position,
            AnalyzerError::ExpectedNumberOrString { position, .. } => *position,
            AnalyzerError::ExpectedString { position, .. } => *position,
            AnalyzerError::ExpectedValueArgumentCount { position, .. } => *position,
            AnalyzerError::TypeConflict {
                actual_statement, ..
            } => actual_statement.position,
            AnalyzerError::UndefinedVariable { identifier } => identifier.position,
            AnalyzerError::UnexpectedIdentifier { identifier } => identifier.position,
            AnalyzerError::UnexectedString { actual } => actual.position,
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
            AnalyzerError::ExpectedNumber { actual, .. } => {
                write!(f, "Expected integer or float, found {}", actual)
            }
            AnalyzerError::ExpectedNumberOrString { actual, .. } => {
                write!(f, "Expected integer, float, or string, found {}", actual)
            }
            AnalyzerError::ExpectedString { actual, .. } => {
                write!(f, "Expected string, found {}", actual)
            }
            AnalyzerError::ExpectedValue { actual, .. } => {
                write!(f, "Expected value, found {}", actual)
            }
            AnalyzerError::ExpectedValueArgumentCount {
                expected, actual, ..
            } => write!(f, "Expected {} value arguments, found {}", expected, actual),
            AnalyzerError::TypeConflict {
                actual_statement,
                actual_type,
                expected,
            } => {
                write!(
                    f,
                    "Expected type {}, found {}, which has type {}",
                    expected, actual_statement, actual_type
                )
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
    fn is_even_wrong_type() {
        let source = "is_even('hello')";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::ExpectedInteger {
                    actual: Node::new(Statement::Constant(Value::string("hello")), (1, 1)),
                    position: (1, 1)
                },
                source
            })
        );
    }

    #[test]
    fn length_no_arguments() {
        let source = "length()";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::ExpectedValueArgumentCount {
                    expected: 1,
                    actual: 0,
                    position: (0, 6)
                },
                source
            })
        );
    }

    #[test]
    fn float_plus_integer() {
        let source = "42.0 + 2";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::TypeConflict {
                    actual_statement: Node::new(Statement::Constant(Value::integer(2)), (7, 8)),
                    actual_type: Type::Integer,
                    expected: Type::Float,
                },
                source
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
            Err(AnalyzerError::ExpectedNumber {
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
            Err(AnalyzerError::ExpectedNumber {
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
