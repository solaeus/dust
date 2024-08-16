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
    ast::{AbstractSyntaxTree, LetStatement, Node, OperatorExpression, Statement},
    parse, Context, DustError, Expression, Identifier, Span, Type,
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
        for statement in &self.abstract_tree.statements {
            self.analyze_statement(statement)?;
        }

        Ok(())
    }

    fn analyze_statement(&mut self, statement: &Statement) -> Result<(), AnalyzerError> {
        match statement {
            Statement::Expression(expression) => self.analyze_expression(expression)?,
            Statement::ExpressionNullified(expression_node) => {
                self.analyze_expression(&expression_node.inner)?;
            }
            Statement::Let(let_statement) => match &let_statement.inner {
                LetStatement::Let { identifier, value } => {
                    let type_option = value.return_type(self.context);

                    if let Some(r#type) = type_option {
                        self.context.set_type(
                            identifier.inner.clone(),
                            r#type,
                            identifier.position,
                        );
                    } else {
                        return Err(AnalyzerError::UndefinedVariable {
                            identifier: identifier.clone(),
                        });
                    }

                    self.analyze_expression(value)?;
                }
                LetStatement::LetMut { identifier, value } => {
                    let type_option = value.return_type(self.context);

                    if let Some(r#type) = type_option {
                        self.context.set_type(
                            identifier.inner.clone(),
                            r#type,
                            identifier.position,
                        );
                    } else {
                        return Err(AnalyzerError::UndefinedVariable {
                            identifier: identifier.clone(),
                        });
                    }

                    self.analyze_expression(value)?;
                }
                LetStatement::LetType {
                    identifier,
                    r#type,
                    value,
                } => todo!(),
                LetStatement::LetMutType {
                    identifier,
                    r#type,
                    value,
                } => todo!(),
            },
            Statement::StructDefinition(_) => {}
        }

        Ok(())
    }

    fn analyze_expression(&mut self, expression: &Expression) -> Result<(), AnalyzerError> {
        match expression {
            Expression::Block(_) => {}
            Expression::Call(_) => {}
            Expression::FieldAccess(_) => {}
            Expression::Grouped(_) => {}
            Expression::Identifier(identifier) => {
                let found = self
                    .context
                    .update_last_position(&identifier.inner, identifier.position);

                if !found {
                    return Err(AnalyzerError::UndefinedVariable {
                        identifier: identifier.clone(),
                    });
                }
            }
            Expression::If(_) => {}
            Expression::List(_) => {}
            Expression::ListIndex(_) => {}
            Expression::Literal(_) => {}
            Expression::Loop(_) => {}
            Expression::Operator(operator_expression) => match operator_expression.inner.as_ref() {
                OperatorExpression::Assignment { assignee, value } => {
                    self.analyze_expression(assignee)?;
                    self.analyze_expression(value)?;
                }
                OperatorExpression::Comparison { left, right, .. } => {
                    self.analyze_expression(left)?;
                    self.analyze_expression(right)?;
                }
                OperatorExpression::CompoundAssignment {
                    assignee, modifier, ..
                } => {
                    self.analyze_expression(assignee)?;
                    self.analyze_expression(modifier)?;
                }
                OperatorExpression::ErrorPropagation(_) => todo!(),
                OperatorExpression::Negation(expression) => {
                    self.analyze_expression(expression)?;
                }
                OperatorExpression::Not(expression) => {
                    self.analyze_expression(expression)?;
                }
                OperatorExpression::Math { left, right, .. } => {
                    self.analyze_expression(left)?;
                    self.analyze_expression(right)?;
                }
                OperatorExpression::Logic { left, right, .. } => {
                    self.analyze_expression(left)?;
                    self.analyze_expression(right)?;
                }
            },
            Expression::Range(_) => {}
            Expression::Struct(_) => {}
            Expression::TupleAccess(_) => {}
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnalyzerError {
    ExpectedBoolean {
        actual: Statement,
    },
    ExpectedIdentifier {
        actual: Statement,
    },
    ExpectedIdentifierOrString {
        actual: Statement,
    },
    ExpectedIntegerOrRange {
        actual: Statement,
    },
    ExpectedList {
        actual: Statement,
    },
    ExpectedMap {
        actual: Statement,
    },
    ExpectedValue {
        actual: Statement,
    },
    ExpectedValueArgumentCount {
        expected: usize,
        actual: usize,
        position: Span,
    },
    IndexOutOfBounds {
        list: Statement,
        index: Statement,
        index_value: usize,
        length: usize,
    },
    TypeConflict {
        actual_statement: Statement,
        actual_type: Type,
        expected: Type,
    },
    UndefinedField {
        identifier: Statement,
        statement: Statement,
    },
    UndefinedType {
        identifier: Node<Identifier>,
    },
    UnexpectedIdentifier {
        identifier: Node<Identifier>,
    },
    UnexectedString {
        actual: Statement,
    },
    UndefinedVariable {
        identifier: Node<Identifier>,
    },
}

impl AnalyzerError {
    pub fn position(&self) -> Span {
        match self {
            AnalyzerError::ExpectedBoolean { actual } => actual.position(),
            AnalyzerError::ExpectedIdentifier { actual } => actual.position(),
            AnalyzerError::ExpectedIdentifierOrString { actual } => actual.position(),
            AnalyzerError::ExpectedIntegerOrRange { actual } => actual.position(),
            AnalyzerError::ExpectedList { actual } => actual.position(),
            AnalyzerError::ExpectedMap { actual } => actual.position(),
            AnalyzerError::ExpectedValue { actual } => actual.position(),
            AnalyzerError::ExpectedValueArgumentCount { position, .. } => *position,
            AnalyzerError::IndexOutOfBounds { index, .. } => index.position(),
            AnalyzerError::TypeConflict {
                actual_statement, ..
            } => actual_statement.position(),
            AnalyzerError::UndefinedField { identifier, .. } => identifier.position(),
            AnalyzerError::UndefinedType { identifier } => identifier.position,
            AnalyzerError::UndefinedVariable { identifier } => identifier.position,
            AnalyzerError::UnexpectedIdentifier { identifier } => identifier.position,
            AnalyzerError::UnexectedString { actual } => actual.position(),
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
    use crate::{Identifier, Value};

    use super::*;

    #[test]
    fn add_assign_wrong_type() {
        let source = "
            a = 1
            a += 1.0
        ";

        assert_eq!(analyze(source), todo!());
    }

    #[test]
    fn subtract_assign_wrong_type() {
        let source = "
            a = 1
            a -= 1.0
        ";

        assert_eq!(analyze(source), todo!());
    }

    #[test]
    fn tuple_struct_with_wrong_field_types() {
        let source = "
            struct Foo(int, float)
            Foo(1, 2)
        ";

        assert_eq!(analyze(source), todo!());
    }

    #[test]
    fn constant_list_index_out_of_bounds() {
        let source = "[1, 2, 3][3]";

        assert_eq!(analyze(source), todo!());
    }

    #[test]
    fn nonexistant_field_identifier() {
        let source = "{ x = 1 }.y";

        assert_eq!(analyze(source), todo!());
    }

    #[test]
    fn nonexistant_field_string() {
        let source = "{ x = 1 }.'y'";

        assert_eq!(analyze(source), todo!());
    }

    #[test]
    fn malformed_list_index() {
        let source = "[1, 2, 3]['foo']";

        assert_eq!(analyze(source), todo!());
    }

    #[test]
    fn malformed_field_access() {
        let source = "{ x = 1 }.0";

        assert_eq!(analyze(source), todo!());
    }

    #[test]
    fn length_no_arguments() {
        let source = "length()";

        assert_eq!(analyze(source), todo!());
    }

    #[test]
    fn float_plus_integer() {
        let source = "42.0 + 2";

        assert_eq!(analyze(source), todo!());
    }

    #[test]
    fn integer_plus_boolean() {
        let source = "42 + true";

        assert_eq!(analyze(source), todo!());
    }

    #[test]
    fn is_even_expects_number() {
        let source = "is_even('hello')";

        assert_eq!(analyze(source), todo!());
    }

    #[test]
    fn is_odd_expects_number() {
        let source = "is_odd('hello')";

        assert_eq!(analyze(source), todo!());
    }

    #[test]
    fn undefined_variable() {
        let source = "foo";

        assert_eq!(analyze(source), todo!());
    }
}
