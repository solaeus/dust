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
    ast::{
        AbstractSyntaxTree, BlockExpression, CallExpression, ElseExpression, FieldAccessExpression,
        IfExpression, LetStatement, ListExpression, ListIndexExpression, LoopExpression,
        MapExpression, Node, OperatorExpression, RangeExpression, Span, Statement,
        StructDefinition, StructExpression, TupleAccessExpression,
    },
    parse, Context, DustError, Expression, Identifier, StructType, Type,
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
        .map_err(|analysis_error| DustError::AnalysisError {
            analysis_error,
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

    pub fn analyze(&mut self) -> Result<(), AnalysisError> {
        for statement in &self.abstract_tree.statements {
            self.analyze_statement(statement)?;
        }

        Ok(())
    }

    fn analyze_statement(&mut self, statement: &Statement) -> Result<(), AnalysisError> {
        match statement {
            Statement::Expression(expression) => self.analyze_expression(expression)?,
            Statement::ExpressionNullified(expression_node) => {
                self.analyze_expression(&expression_node.inner)?;
            }
            Statement::Let(let_statement) => match &let_statement.inner {
                LetStatement::Let { identifier, value }
                | LetStatement::LetMut { identifier, value } => {
                    self.analyze_expression(value)?;

                    let r#type = value.return_type(self.context);

                    if let Some(r#type) = r#type {
                        self.context.set_variable_type(
                            identifier.inner.clone(),
                            r#type,
                            identifier.position,
                        );
                    } else {
                        return Err(AnalysisError::ExpectedValueFromExpression {
                            actual: value.clone(),
                        });
                    }
                }
                LetStatement::LetType { .. } => todo!(),
                LetStatement::LetMutType { .. } => todo!(),
            },
            Statement::StructDefinition(struct_definition) => {
                let (name, struct_type) = match &struct_definition.inner {
                    StructDefinition::Unit { name } => (
                        name,
                        Type::Struct(StructType::Unit {
                            name: name.inner.clone(),
                        }),
                    ),
                    StructDefinition::Tuple { name, items } => {
                        let fields = items.iter().map(|item| item.inner.clone()).collect();

                        (
                            name,
                            Type::Struct(StructType::Tuple {
                                name: name.inner.clone(),
                                fields,
                            }),
                        )
                    }
                    StructDefinition::Fields { name, fields } => {
                        let fields = fields
                            .iter()
                            .map(|(identifier, r#type)| {
                                (identifier.inner.clone(), r#type.inner.clone())
                            })
                            .collect();

                        (
                            name,
                            Type::Struct(StructType::Fields {
                                name: name.inner.clone(),
                                fields,
                            }),
                        )
                    }
                };

                todo!("Set constructor")
            }
        }

        Ok(())
    }

    fn analyze_expression(&mut self, expression: &Expression) -> Result<(), AnalysisError> {
        match expression {
            Expression::Block(block_expression) => self.analyze_block(&block_expression.inner)?,
            Expression::Call(call_expression) => {
                let CallExpression { invoker, arguments } = call_expression.inner.as_ref();

                self.analyze_expression(invoker)?;

                for argument in arguments {
                    self.analyze_expression(argument)?;
                }
            }
            Expression::FieldAccess(field_access_expression) => {
                let FieldAccessExpression { container, field } =
                    field_access_expression.inner.as_ref();

                self.context
                    .update_last_position(&field.inner, field.position);
                self.analyze_expression(container)?;
            }
            Expression::Grouped(expression) => {
                self.analyze_expression(expression.inner.as_ref())?;
            }
            Expression::Identifier(identifier) => {
                self.context
                    .update_last_position(&identifier.inner, identifier.position);
            }
            Expression::If(if_expression) => self.analyze_if(&if_expression.inner)?,
            Expression::List(list_expression) => match list_expression.inner.as_ref() {
                ListExpression::AutoFill {
                    repeat_operand,
                    length_operand,
                } => {
                    self.analyze_expression(repeat_operand)?;
                    self.analyze_expression(length_operand)?;
                }
                ListExpression::Ordered(expressions) => {
                    for expression in expressions {
                        self.analyze_expression(expression)?;
                    }
                }
            },
            Expression::ListIndex(list_index_expression) => {
                let ListIndexExpression { list, index } = list_index_expression.inner.as_ref();

                self.analyze_expression(list)?;
                self.analyze_expression(index)?;
            }
            Expression::Literal(_) => {
                // Literals don't need to be analyzed
            }
            Expression::Loop(loop_expression) => match loop_expression.inner.as_ref() {
                LoopExpression::Infinite { block } => self.analyze_block(&block.inner)?,
                LoopExpression::While { condition, block } => {
                    self.analyze_expression(condition)?;
                    self.analyze_block(&block.inner)?;
                }
                LoopExpression::For {
                    iterator, block, ..
                } => {
                    self.analyze_expression(iterator)?;
                    self.analyze_block(&block.inner)?;
                }
            },
            Expression::Map(map_expression) => {
                let MapExpression { pairs } = map_expression.inner.as_ref();

                for (_, expression) in pairs {
                    self.analyze_expression(expression)?;
                }
            }
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

                    let expected_type = assignee.return_type(self.context);
                    let actual_type = modifier.return_type(self.context);

                    if expected_type.is_none() {
                        return Err(AnalysisError::ExpectedValueFromExpression {
                            actual: assignee.clone(),
                        });
                    }

                    if actual_type.is_none() {
                        return Err(AnalysisError::ExpectedValueFromExpression {
                            actual: modifier.clone(),
                        });
                    }

                    if let (Some(expected_type), Some(actual_type)) = (expected_type, actual_type) {
                        expected_type.check(&actual_type).map_err(|type_conflct| {
                            AnalysisError::TypeConflict {
                                actual_expression: modifier.clone(),
                                actual_type,
                                expected: expected_type,
                            }
                        })?;
                    }
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
            Expression::Range(range_expression) => match range_expression.inner.as_ref() {
                RangeExpression::Exclusive { start, end } => {
                    self.analyze_expression(start)?;
                    self.analyze_expression(end)?;
                }
                RangeExpression::Inclusive { start, end } => {
                    self.analyze_expression(start)?;
                    self.analyze_expression(end)?;
                }
            },
            Expression::Struct(struct_expression) => match struct_expression.inner.as_ref() {
                StructExpression::Unit { name } => {
                    todo!("Update constructor position");
                }
                StructExpression::Fields { name, fields } => {
                    todo!("Update constructor position");

                    // for (_, expression) in fields {
                    // self.analyze_expression(expression)?;
                    // }
                }
            },
            Expression::TupleAccess(tuple_access) => {
                let TupleAccessExpression { tuple, .. } = tuple_access.inner.as_ref();

                self.analyze_expression(tuple)?;
            }
        }

        Ok(())
    }

    fn analyze_block(&mut self, block_expression: &BlockExpression) -> Result<(), AnalysisError> {
        match block_expression {
            BlockExpression::Async(statements) => {
                for statement in statements {
                    self.analyze_statement(statement)?;
                }
            }
            BlockExpression::Sync(statements) => {
                for statement in statements {
                    self.analyze_statement(statement)?;
                }
            }
        }

        Ok(())
    }

    fn analyze_if(&mut self, if_expression: &IfExpression) -> Result<(), AnalysisError> {
        match if_expression {
            IfExpression::If {
                condition,
                if_block,
            } => {
                self.analyze_expression(condition)?;
                self.analyze_block(&if_block.inner)?;
            }
            IfExpression::IfElse {
                condition,
                if_block,
                r#else,
            } => {
                self.analyze_expression(condition)?;
                self.analyze_block(&if_block.inner)?;

                match r#else {
                    ElseExpression::Block(block_expression) => {
                        self.analyze_block(&block_expression.inner)?;
                    }
                    ElseExpression::If(if_expression) => {
                        self.analyze_if(&if_expression.inner)?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnalysisError {
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
    ExpectedValueFromStatement {
        actual: Statement,
    },
    ExpectedValueFromExpression {
        actual: Expression,
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
        actual_expression: Expression,
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

impl AnalysisError {
    pub fn position(&self) -> Span {
        match self {
            AnalysisError::ExpectedBoolean { actual } => actual.position(),
            AnalysisError::ExpectedIdentifier { actual } => actual.position(),
            AnalysisError::ExpectedIdentifierOrString { actual } => actual.position(),
            AnalysisError::ExpectedIntegerOrRange { actual } => actual.position(),
            AnalysisError::ExpectedList { actual } => actual.position(),
            AnalysisError::ExpectedMap { actual } => actual.position(),
            AnalysisError::ExpectedValueFromExpression { actual } => actual.position(),
            AnalysisError::ExpectedValueFromStatement { actual } => actual.position(),
            AnalysisError::ExpectedValueArgumentCount { position, .. } => *position,
            AnalysisError::IndexOutOfBounds { index, .. } => index.position(),
            AnalysisError::TypeConflict {
                actual_expression, ..
            } => actual_expression.position(),
            AnalysisError::UndefinedField { identifier, .. } => identifier.position(),
            AnalysisError::UndefinedType { identifier } => identifier.position,
            AnalysisError::UndefinedVariable { identifier } => identifier.position,
            AnalysisError::UnexpectedIdentifier { identifier } => identifier.position,
            AnalysisError::UnexectedString { actual } => actual.position(),
        }
    }
}

impl Error for AnalysisError {}

impl Display for AnalysisError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            AnalysisError::ExpectedBoolean { actual, .. } => {
                write!(f, "Expected boolean, found {}", actual)
            }
            AnalysisError::ExpectedIdentifier { actual, .. } => {
                write!(f, "Expected identifier, found {}", actual)
            }
            AnalysisError::ExpectedIdentifierOrString { actual } => {
                write!(f, "Expected identifier or string, found {}", actual)
            }
            AnalysisError::ExpectedIntegerOrRange { actual, .. } => {
                write!(f, "Expected integer or range, found {}", actual)
            }
            AnalysisError::ExpectedList { actual } => write!(f, "Expected list, found {}", actual),
            AnalysisError::ExpectedMap { actual } => write!(f, "Expected map, found {}", actual),
            AnalysisError::ExpectedValueFromExpression { actual, .. } => {
                write!(
                    f,
                    "Expected expression to produce a value, found {}",
                    actual
                )
            }
            AnalysisError::ExpectedValueFromStatement { actual, .. } => {
                write!(f, "Expected statement to produce a value, found {}", actual)
            }
            AnalysisError::ExpectedValueArgumentCount {
                expected, actual, ..
            } => write!(f, "Expected {} value arguments, found {}", expected, actual),
            AnalysisError::IndexOutOfBounds {
                list,
                index_value,
                length,
                ..
            } => write!(
                f,
                "Index {} out of bounds for list {} with length {}",
                index_value, list, length
            ),
            AnalysisError::TypeConflict {
                actual_expression: actual_statement,
                actual_type,
                expected,
            } => {
                write!(
                    f,
                    "Expected type {}, found {}, which has type {}",
                    expected, actual_statement, actual_type
                )
            }
            AnalysisError::UndefinedField {
                identifier,
                statement: map,
            } => {
                write!(f, "Undefined field {} in map {}", identifier, map)
            }
            AnalysisError::UndefinedType { identifier } => {
                write!(f, "Undefined type {}", identifier)
            }
            AnalysisError::UndefinedVariable { identifier } => {
                write!(f, "Undefined variable {}", identifier)
            }
            AnalysisError::UnexpectedIdentifier { identifier, .. } => {
                write!(f, "Unexpected identifier {}", identifier)
            }
            AnalysisError::UnexectedString { actual, .. } => {
                write!(f, "Unexpected string {}", actual)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn add_assign_wrong_type() {
        let source = "
            let mut a = 1;
            a += 1.0
        ";

        assert_eq!(
            analyze(source),
            Err(DustError::AnalysisError {
                analysis_error: AnalysisError::TypeConflict {
                    actual_expression: Expression::literal(1.0, (45, 48)),
                    actual_type: Type::Float,
                    expected: Type::Integer,
                },
                source,
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
            Err(DustError::AnalysisError {
                analysis_error: AnalysisError::TypeConflict {
                    actual_expression: Expression::literal(1.0, (45, 48)),
                    actual_type: Type::Float,
                    expected: Type::Integer,
                },
                source,
            })
        );
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
