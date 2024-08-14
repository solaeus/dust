//! Tools for analyzing an abstract syntax tree and catching errors before running the virtual
//! machine.
//!
//! This module provides two anlysis options:
//! - `analyze` convenience function, which takes a string input
//! - `Analyzer` struct, which borrows an abstract syntax tree and a context
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use crate::{
    abstract_tree::{BinaryOperator, UnaryOperator},
    parse, AbstractSyntaxTree, Context, DustError, Identifier, Node, Span, Statement,
    StructDefinition, StructType, Type,
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
    let context = Context::new();
    let mut analyzer = Analyzer::new(&abstract_tree, &context);

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
    context: &'a Context,
}

impl<'a> Analyzer<'a> {
    pub fn new(abstract_tree: &'a AbstractSyntaxTree, context: &'a Context) -> Self {
        Self {
            abstract_tree,
            context,
        }
    }

    pub fn analyze(&mut self) -> Result<(), AnalyzerError> {
        for node in &self.abstract_tree.statements {
            self.analyze_statement(node)?;
        }

        Ok(())
    }

    fn analyze_statement(&mut self, _: &Node<Statement>) -> Result<(), AnalyzerError> {
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnalyzerError {
    ExpectedBoolean {
        actual: Node<Statement>,
    },
    ExpectedIdentifier {
        actual: Node<Statement>,
    },
    ExpectedIdentifierOrString {
        actual: Node<Statement>,
    },
    ExpectedIntegerOrRange {
        actual: Node<Statement>,
    },
    ExpectedList {
        actual: Node<Statement>,
    },
    ExpectedMap {
        actual: Node<Statement>,
    },
    ExpectedValue {
        actual: Node<Statement>,
    },
    ExpectedValueArgumentCount {
        expected: usize,
        actual: usize,
        position: Span,
    },
    IndexOutOfBounds {
        list: Node<Statement>,
        index: Node<Statement>,
        index_value: usize,
        length: usize,
    },
    TypeConflict {
        actual_statement: Node<Statement>,
        actual_type: Type,
        expected: Type,
    },
    UndefinedField {
        identifier: Node<Statement>,
        statement: Node<Statement>,
    },
    UndefinedType {
        identifier: Node<Identifier>,
    },
    UnexpectedIdentifier {
        identifier: Node<Statement>,
    },
    UnexectedString {
        actual: Node<Statement>,
    },
    UndefinedVariable {
        identifier: Node<Statement>,
    },
}

impl AnalyzerError {
    pub fn position(&self) -> Span {
        match self {
            AnalyzerError::ExpectedBoolean { actual, .. } => actual.position,
            AnalyzerError::ExpectedIdentifier { actual, .. } => actual.position,
            AnalyzerError::ExpectedIdentifierOrString { actual } => actual.position,
            AnalyzerError::ExpectedIntegerOrRange { actual, .. } => actual.position,
            AnalyzerError::ExpectedList { actual } => actual.position,
            AnalyzerError::ExpectedMap { actual } => actual.position,
            AnalyzerError::ExpectedValue { actual } => actual.position,
            AnalyzerError::ExpectedValueArgumentCount { position, .. } => *position,
            AnalyzerError::IndexOutOfBounds { list, index, .. } => {
                (list.position.0, index.position.1)
            }
            AnalyzerError::TypeConflict {
                actual_statement, ..
            } => actual_statement.position,
            AnalyzerError::UndefinedField { identifier, .. } => identifier.position,
            AnalyzerError::UndefinedType { identifier } => identifier.position,
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
            AnalyzerError::ExpectedIdentifier { actual, .. } => {
                write!(f, "Expected identifier, found {}", actual)
            }
            AnalyzerError::ExpectedIdentifierOrString { actual } => {
                write!(f, "Expected identifier or string, found {}", actual)
            }
            AnalyzerError::ExpectedIntegerOrRange { actual, .. } => {
                write!(f, "Expected integer or range, found {}", actual)
            }
            AnalyzerError::ExpectedList { actual } => write!(f, "Expected list, found {}", actual),
            AnalyzerError::ExpectedMap { actual } => write!(f, "Expected map, found {}", actual),
            AnalyzerError::ExpectedValue { actual, .. } => {
                write!(f, "Expected value, found {}", actual)
            }
            AnalyzerError::ExpectedValueArgumentCount {
                expected, actual, ..
            } => write!(f, "Expected {} value arguments, found {}", expected, actual),
            AnalyzerError::IndexOutOfBounds {
                list,
                index_value,
                length,
                ..
            } => write!(
                f,
                "Index {} out of bounds for list {} with length {}",
                index_value, list, length
            ),
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
            AnalyzerError::UndefinedField {
                identifier,
                statement: map,
            } => {
                write!(f, "Undefined field {} in map {}", identifier, map)
            }
            AnalyzerError::UndefinedType { identifier } => {
                write!(f, "Undefined type {}", identifier)
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
    use crate::{AssignmentOperator, Identifier, Value};

    use super::*;

    #[test]
    fn add_assign_wrong_type() {
        let source = "
            a = 1
            a += 1.0
        ";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::TypeConflict {
                    actual_statement: Node::new(
                        Statement::Assignment {
                            identifier: Node::new(Identifier::new("a"), (31, 32)),
                            operator: Node::new(AssignmentOperator::AddAssign, (33, 35)),
                            value: Box::new(Node::new(
                                Statement::Constant(Value::float(1.0)),
                                (38, 41)
                            ))
                        },
                        (31, 32)
                    ),
                    actual_type: Type::Integer,
                    expected: Type::Float
                },
                source
            })
        );
    }

    #[test]
    fn subtract_assign_wrong_type() {
        let source = "
            a = 1
            a -= 1.0
        ";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::TypeConflict {
                    actual_statement: Node::new(
                        Statement::Assignment {
                            identifier: Node::new(Identifier::new("a"), (31, 32)),
                            operator: Node::new(AssignmentOperator::SubtractAssign, (33, 37)),
                            value: Box::new(Node::new(
                                Statement::Constant(Value::float(1.0)),
                                (40, 43)
                            ))
                        },
                        (31, 32)
                    ),
                    actual_type: Type::Integer,
                    expected: Type::Float
                },
                source
            })
        );
    }

    #[test]
    fn tuple_struct_with_wrong_field_types() {
        let source = "
            struct Foo(int, float)
            Foo(1, 2)
        ";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::TypeConflict {
                    actual_statement: Node::new(Statement::Constant(Value::integer(2)), (55, 56)),
                    actual_type: Type::Integer,
                    expected: Type::Float
                },
                source
            })
        );
    }

    #[test]
    fn constant_list_index_out_of_bounds() {
        let source = "[1, 2, 3][3]";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::IndexOutOfBounds {
                    list: Node::new(
                        Statement::List(vec![
                            Node::new(Statement::Constant(Value::integer(1)), (1, 2)),
                            Node::new(Statement::Constant(Value::integer(2)), (4, 5)),
                            Node::new(Statement::Constant(Value::integer(3)), (7, 8)),
                        ]),
                        (0, 9)
                    ),
                    index: Node::new(Statement::Constant(Value::integer(3)), (10, 11)),
                    index_value: 3,
                    length: 3
                },
                source
            })
        );
    }

    #[test]
    fn nonexistant_field_identifier() {
        let source = "{ x = 1 }.y";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::UndefinedField {
                    identifier: Node::new(Statement::Identifier(Identifier::new("y")), (10, 11)),
                    statement: Node::new(
                        Statement::Map(vec![(
                            Node::new(Identifier::new("x"), (2, 3)),
                            Node::new(Statement::Constant(Value::integer(1)), (6, 7))
                        )]),
                        (0, 9)
                    )
                },
                source
            })
        );
    }

    #[test]
    fn nonexistant_field_string() {
        let source = "{ x = 1 }.'y'";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::UndefinedField {
                    identifier: Node::new(Statement::Constant(Value::string("y")), (10, 13)),
                    statement: Node::new(
                        Statement::Map(vec![(
                            Node::new(Identifier::new("x"), (2, 3)),
                            Node::new(Statement::Constant(Value::integer(1)), (6, 7))
                        )]),
                        (0, 9)
                    )
                },
                source
            })
        );
    }

    #[test]
    fn malformed_list_index() {
        let source = "[1, 2, 3]['foo']";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::ExpectedIntegerOrRange {
                    actual: Node::new(Statement::Constant(Value::string("foo")), (10, 15)),
                },
                source
            })
        );
    }

    #[test]
    fn malformed_field_access() {
        let source = "{ x = 1 }.0";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::ExpectedIdentifierOrString {
                    actual: Node::new(Statement::Constant(Value::integer(0)), (10, 11)),
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
        let source = "42 + true";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::TypeConflict {
                    actual_statement: Node::new(Statement::Constant(Value::boolean(true)), (5, 9)),
                    actual_type: Type::Boolean,
                    expected: Type::Integer,
                },
                source
            })
        )
    }

    #[test]
    fn is_even_expects_number() {
        let source = "is_even('hello')";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::TypeConflict {
                    actual_statement: Node::new(
                        Statement::Constant(Value::string("hello")),
                        (8, 15)
                    ),
                    actual_type: Type::String,
                    expected: Type::Number,
                },
                source
            })
        );
    }

    #[test]
    fn is_odd_expects_number() {
        let source = "is_odd('hello')";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::TypeConflict {
                    actual_statement: Node::new(
                        Statement::Constant(Value::string("hello")),
                        (7, 14)
                    ),
                    actual_type: Type::String,
                    expected: Type::Number,
                },
                source
            })
        );
    }

    #[test]
    fn undefined_variable() {
        let source = "foo";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalyzerError {
                analyzer_error: AnalyzerError::UndefinedVariable {
                    identifier: Node::new(Statement::Identifier(Identifier::new("foo")), (0, 3)),
                },
                source
            })
        );
    }
}
