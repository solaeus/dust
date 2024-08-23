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
        AbstractSyntaxTree, AstError, BlockExpression, CallExpression, ElseExpression,
        FieldAccessExpression, IfExpression, LetStatement, ListExpression, ListIndexExpression,
        LiteralExpression, LoopExpression, MapExpression, Node, OperatorExpression,
        PrimitiveValueExpression, RangeExpression, Span, Statement, StructDefinition,
        StructExpression, TupleAccessExpression,
    },
    core_library, parse, Context, ContextError, DustError, Expression, Identifier, RangeableType,
    StructType, Type,
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
    let context = core_library().create_child();
    let mut analyzer = Analyzer::new(&abstract_tree, context);

    analyzer.analyze();

    if analyzer.errors.is_empty() {
        Ok(())
    } else {
        Err(DustError::analysis(analyzer.errors, source))
    }
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
    context: Context,
    pub errors: Vec<AnalysisError>,
}

impl<'a> Analyzer<'a> {
    pub fn new(abstract_tree: &'a AbstractSyntaxTree, context: Context) -> Self {
        Self {
            abstract_tree,
            context,
            errors: Vec::new(),
        }
    }

    pub fn analyze(&mut self) {
        for statement in &self.abstract_tree.statements {
            self.analyze_statement(statement);
        }
    }

    fn analyze_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Expression(expression) => self.analyze_expression(expression),
            Statement::ExpressionNullified(expression_node) => {
                self.analyze_expression(&expression_node.inner);
            }
            Statement::Let(let_statement) => match &let_statement.inner {
                LetStatement::Let { identifier, value }
                | LetStatement::LetMut { identifier, value } => {
                    let r#type = match value.return_type(&self.context) {
                        Ok(type_option) => type_option,
                        Err(ast_error) => {
                            self.errors.push(AnalysisError::AstError(ast_error));

                            None
                        }
                    };

                    if let Some(r#type) = r#type {
                        let set_type = self.context.set_variable_type(
                            identifier.inner.clone(),
                            r#type.clone(),
                            identifier.position,
                        );

                        if let Err(context_error) = set_type {
                            self.errors.push(AnalysisError::ContextError {
                                error: context_error,
                                position: identifier.position,
                            });
                        }
                    } else {
                        self.errors
                            .push(AnalysisError::LetExpectedValueFromStatement {
                                actual: value.clone(),
                            });
                    }

                    self.analyze_expression(value);
                }
                LetStatement::LetType { .. } => todo!(),
                LetStatement::LetMutType { .. } => todo!(),
            },
            Statement::StructDefinition(struct_definition) => {
                let set_constructor_type = match &struct_definition.inner {
                    StructDefinition::Unit { name } => self.context.set_constructor_type(
                        name.inner.clone(),
                        StructType::Unit {
                            name: name.inner.clone(),
                        },
                        name.position,
                    ),
                    StructDefinition::Tuple { name, items } => {
                        let fields = items.iter().map(|item| item.inner.clone()).collect();

                        self.context.set_constructor_type(
                            name.inner.clone(),
                            StructType::Tuple {
                                name: name.inner.clone(),
                                fields,
                            },
                            name.position,
                        )
                    }
                    StructDefinition::Fields { name, fields } => {
                        let fields = fields
                            .iter()
                            .map(|(identifier, r#type)| {
                                (identifier.inner.clone(), r#type.inner.clone())
                            })
                            .collect();

                        self.context.set_constructor_type(
                            name.inner.clone(),
                            StructType::Fields {
                                name: name.inner.clone(),
                                fields,
                            },
                            name.position,
                        )
                    }
                };

                if let Err(context_error) = set_constructor_type {
                    self.errors.push(AnalysisError::ContextError {
                        error: context_error,
                        position: struct_definition.position,
                    });
                }
            }
        }
    }

    fn analyze_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Block(block_expression) => self.analyze_block(&block_expression.inner),
            Expression::Break(break_node) => {
                if let Some(expression) = &break_node.inner {
                    self.analyze_expression(expression);
                }
            }
            Expression::Call(call_expression) => {
                let CallExpression { invoker, arguments } = call_expression.inner.as_ref();

                self.analyze_expression(invoker);

                for argument in arguments {
                    self.analyze_expression(argument);
                }
            }
            Expression::FieldAccess(field_access_expression) => {
                let FieldAccessExpression { container, .. } =
                    field_access_expression.inner.as_ref();

                self.analyze_expression(container);
            }
            Expression::Grouped(expression) => {
                self.analyze_expression(expression.inner.as_ref());
            }
            Expression::Identifier(identifier) => {
                let found = self
                    .context
                    .update_last_position(&identifier.inner, identifier.position)
                    .map_err(|error| {
                        self.errors.push(AnalysisError::ContextError {
                            error,
                            position: identifier.position,
                        })
                    });

                if let Ok(false) = found {
                    self.errors.push(AnalysisError::UndefinedVariable {
                        identifier: identifier.clone(),
                    });
                }
            }
            Expression::If(if_expression) => self.analyze_if(&if_expression.inner),
            Expression::List(list_expression) => match list_expression.inner.as_ref() {
                ListExpression::AutoFill {
                    repeat_operand,
                    length_operand,
                } => {
                    self.analyze_expression(repeat_operand);
                    self.analyze_expression(length_operand);
                }
                ListExpression::Ordered(expressions) => {
                    for expression in expressions {
                        self.analyze_expression(expression);
                    }
                }
            },
            Expression::ListIndex(list_index_expression) => {
                let ListIndexExpression { list, index } = list_index_expression.inner.as_ref();

                self.analyze_expression(list);
                self.analyze_expression(index);

                let list_type = match list.return_type(&self.context) {
                    Ok(Some(r#type)) => r#type,
                    Ok(None) => {
                        self.errors
                            .push(AnalysisError::ExpectedValueFromExpression {
                                expression: list.clone(),
                            });

                        return;
                    }
                    Err(ast_error) => {
                        self.errors.push(AnalysisError::AstError(ast_error));

                        return;
                    }
                };
                let index_type = match list.return_type(&self.context) {
                    Ok(Some(r#type)) => r#type,
                    Ok(None) => {
                        self.errors
                            .push(AnalysisError::ExpectedValueFromExpression {
                                expression: list.clone(),
                            });

                        return;
                    }
                    Err(ast_error) => {
                        self.errors.push(AnalysisError::AstError(ast_error));

                        return;
                    }
                };
                let literal_type = if let Expression::Literal(Node { inner, .. }) = index {
                    Some(inner.as_ref().clone())
                } else {
                    None
                };

                if let Some(LiteralExpression::Primitive(PrimitiveValueExpression::Integer(
                    integer,
                ))) = literal_type
                {
                    if integer < 0 {
                        self.errors.push(AnalysisError::NegativeIndex {
                            index: index.clone(),
                            index_value: integer,
                            list: list.clone(),
                        });
                    }
                }

                if let Type::List { length, .. } = list_type {
                    if let Some(LiteralExpression::Primitive(PrimitiveValueExpression::Integer(
                        integer,
                    ))) = literal_type
                    {
                        if integer >= length as i64 {
                            self.errors.push(AnalysisError::IndexOutOfBounds {
                                index: index.clone(),
                                length,
                                list: list.clone(),
                                index_value: integer,
                            });
                        }
                    }
                }

                if let Type::String {
                    length: Some(length),
                } = list_type
                {
                    if let Some(LiteralExpression::Primitive(PrimitiveValueExpression::Integer(
                        integer,
                    ))) = literal_type
                    {
                        if integer >= length as i64 {
                            self.errors.push(AnalysisError::IndexOutOfBounds {
                                index: index.clone(),
                                length,
                                list: list.clone(),
                                index_value: integer,
                            });
                        }
                    }
                }
            }
            Expression::Literal(_) => {
                // Literals don't need to be analyzed
            }
            Expression::Loop(loop_expression) => match loop_expression.inner.as_ref() {
                LoopExpression::Infinite { block } => self.analyze_block(&block.inner),
                LoopExpression::While { condition, block } => {
                    self.analyze_expression(condition);
                    self.analyze_block(&block.inner);
                }
                LoopExpression::For {
                    iterator, block, ..
                } => {
                    self.analyze_expression(iterator);
                    self.analyze_block(&block.inner);
                }
            },
            Expression::Map(map_expression) => {
                let MapExpression { pairs } = map_expression.inner.as_ref();

                for (_, expression) in pairs {
                    self.analyze_expression(expression);
                }
            }
            Expression::Operator(operator_expression) => match operator_expression.inner.as_ref() {
                OperatorExpression::Assignment { assignee, value } => {
                    self.analyze_expression(assignee);
                    self.analyze_expression(value);
                }
                OperatorExpression::Comparison { left, right, .. } => {
                    self.analyze_expression(left);
                    self.analyze_expression(right);
                }
                OperatorExpression::CompoundAssignment {
                    assignee, modifier, ..
                } => {
                    self.analyze_expression(assignee);
                    self.analyze_expression(modifier);

                    let (expected_type, actual_type) = match (
                        assignee.return_type(&self.context),
                        modifier.return_type(&self.context),
                    ) {
                        (Ok(Some(expected_type)), Ok(Some(actual_type))) => {
                            (expected_type, actual_type)
                        }
                        (Ok(None), Ok(None)) => {
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: assignee.clone(),
                                });
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: modifier.clone(),
                                });
                            return;
                        }
                        (Ok(None), _) => {
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: assignee.clone(),
                                });
                            return;
                        }
                        (_, Ok(None)) => {
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: modifier.clone(),
                                });
                            return;
                        }
                        (Err(ast_error), _) => {
                            self.errors.push(AnalysisError::AstError(ast_error));
                            return;
                        }
                        (_, Err(ast_error)) => {
                            self.errors.push(AnalysisError::AstError(ast_error));
                            return;
                        }
                    };

                    if actual_type != expected_type {
                        self.errors.push(AnalysisError::ExpectedType {
                            expected: expected_type,
                            actual: actual_type,
                            actual_expression: modifier.clone(),
                        });
                    }
                }
                OperatorExpression::ErrorPropagation(_) => todo!(),
                OperatorExpression::Negation(expression) => {
                    self.analyze_expression(expression);
                }
                OperatorExpression::Not(expression) => {
                    self.analyze_expression(expression);
                }
                OperatorExpression::Math { left, right, .. } => {
                    self.analyze_expression(left);
                    self.analyze_expression(right);

                    let (left_type, right_type) = match (
                        left.return_type(&self.context),
                        right.return_type(&self.context),
                    ) {
                        (Ok(Some(left_type)), Ok(Some(right_type))) => (left_type, right_type),
                        (Ok(None), Ok(None)) => {
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: left.clone(),
                                });
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: right.clone(),
                                });
                            return;
                        }
                        (Ok(None), _) => {
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: left.clone(),
                                });
                            return;
                        }
                        (_, Ok(None)) => {
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: right.clone(),
                                });
                            return;
                        }
                        (Err(ast_error), _) => {
                            self.errors.push(AnalysisError::AstError(ast_error));
                            return;
                        }
                        (_, Err(ast_error)) => {
                            self.errors.push(AnalysisError::AstError(ast_error));
                            return;
                        }
                    };

                    match left_type {
                        Type::Integer => {
                            if right_type != Type::Integer {
                                self.errors.push(AnalysisError::ExpectedType {
                                    expected: Type::Integer,
                                    actual: right_type,
                                    actual_expression: right.clone(),
                                });
                            }
                        }
                        Type::Float => {
                            if right_type != Type::Float {
                                self.errors.push(AnalysisError::ExpectedType {
                                    expected: Type::Float,
                                    actual: right_type,
                                    actual_expression: right.clone(),
                                });
                            }
                        }

                        Type::String { .. } => {
                            if let Type::String { .. } = right_type {
                            } else {
                                self.errors.push(AnalysisError::ExpectedType {
                                    expected: Type::String { length: None },
                                    actual: right_type,
                                    actual_expression: right.clone(),
                                });
                            }
                        }
                        _ => {
                            self.errors.push(AnalysisError::ExpectedTypeMultiple {
                                expected: vec![
                                    Type::Float,
                                    Type::Integer,
                                    Type::String { length: None },
                                ],
                                actual: left_type,
                                actual_expression: left.clone(),
                            });
                        }
                    }
                }
                OperatorExpression::Logic { left, right, .. } => {
                    self.analyze_expression(left);
                    self.analyze_expression(right);

                    let (left_type, right_type) = match (
                        left.return_type(&self.context),
                        right.return_type(&self.context),
                    ) {
                        (Ok(Some(left_type)), Ok(Some(right_type))) => (left_type, right_type),
                        (Ok(None), Ok(None)) => {
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: left.clone(),
                                });
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: right.clone(),
                                });
                            return;
                        }
                        (Ok(None), _) => {
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: left.clone(),
                                });
                            return;
                        }
                        (_, Ok(None)) => {
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: right.clone(),
                                });
                            return;
                        }
                        (Err(ast_error), _) => {
                            self.errors.push(AnalysisError::AstError(ast_error));
                            return;
                        }
                        (_, Err(ast_error)) => {
                            self.errors.push(AnalysisError::AstError(ast_error));
                            return;
                        }
                    };

                    if left_type != right_type {
                        self.errors.push(AnalysisError::ExpectedType {
                            expected: left_type,
                            actual: right_type,
                            actual_expression: right.clone(),
                        });
                    }
                }
            },
            Expression::Range(range_expression) => match range_expression.inner.as_ref() {
                RangeExpression::Exclusive { start, end } => {
                    self.analyze_expression(start);
                    self.analyze_expression(end);
                }
                RangeExpression::Inclusive { start, end } => {
                    self.analyze_expression(start);
                    self.analyze_expression(end);
                }
            },
            Expression::Struct(struct_expression) => match struct_expression.inner.as_ref() {
                StructExpression::Fields { name, fields } => {
                    let update_position = self
                        .context
                        .update_last_position(&name.inner, name.position);

                    if let Err(error) = update_position {
                        self.errors.push(AnalysisError::ContextError {
                            error,
                            position: name.position,
                        });

                        return;
                    }

                    for (_, expression) in fields {
                        self.analyze_expression(expression);
                    }
                }
            },
            Expression::TupleAccess(tuple_access) => {
                let TupleAccessExpression { tuple, .. } = tuple_access.inner.as_ref();

                self.analyze_expression(tuple);
            }
        }
    }

    fn analyze_block(&mut self, block_expression: &BlockExpression) {
        match block_expression {
            BlockExpression::Async(statements) => {
                for statement in statements {
                    self.analyze_statement(statement);
                }
            }
            BlockExpression::Sync(statements) => {
                for statement in statements {
                    self.analyze_statement(statement);
                }
            }
        }
    }

    fn analyze_if(&mut self, if_expression: &IfExpression) {
        match if_expression {
            IfExpression::If {
                condition,
                if_block,
            } => {
                self.analyze_expression(condition);
                self.analyze_block(&if_block.inner);
            }
            IfExpression::IfElse {
                condition,
                if_block,
                r#else,
            } => {
                self.analyze_expression(condition);
                self.analyze_block(&if_block.inner);

                match r#else {
                    ElseExpression::Block(block_expression) => {
                        self.analyze_block(&block_expression.inner);
                    }
                    ElseExpression::If(if_expression) => {
                        self.analyze_if(&if_expression.inner);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnalysisError {
    AstError(AstError),
    ContextError {
        error: ContextError,
        position: Span,
    },
    ExpectedType {
        expected: Type,
        actual: Type,
        actual_expression: Expression,
    },
    ExpectedTypeMultiple {
        expected: Vec<Type>,
        actual: Type,
        actual_expression: Expression,
    },
    ExpectedIdentifier {
        actual: Expression,
    },
    ExpectedIdentifierOrString {
        actual: Expression,
    },
    LetExpectedValueFromStatement {
        actual: Expression,
    },
    ExpectedValueFromExpression {
        expression: Expression,
    },
    ExpectedValueArgumentCount {
        expected: usize,
        actual: usize,
        position: Span,
    },
    IndexOutOfBounds {
        list: Expression,
        index: Expression,
        index_value: i64,
        length: usize,
    },
    NegativeIndex {
        list: Expression,
        index: Expression,
        index_value: i64,
    },
    TypeConflict {
        actual_expression: Expression,
        actual_type: Type,
        expected: Type,
    },
    UndefinedField {
        identifier: Expression,
        expression: Expression,
    },
    UndefinedType {
        identifier: Node<Identifier>,
    },
    UnexpectedIdentifier {
        identifier: Node<Identifier>,
    },
    UnexectedString {
        actual: Expression,
    },
    UndefinedVariable {
        identifier: Node<Identifier>,
    },
}

impl From<AstError> for AnalysisError {
    fn from(v: AstError) -> Self {
        Self::AstError(v)
    }
}

impl AnalysisError {
    pub fn position(&self) -> Span {
        match self {
            AnalysisError::AstError(ast_error) => ast_error.position(),
            AnalysisError::ContextError { position, .. } => *position,
            AnalysisError::ExpectedType {
                actual_expression, ..
            } => actual_expression.position(),
            AnalysisError::ExpectedTypeMultiple {
                actual_expression, ..
            } => actual_expression.position(),
            AnalysisError::ExpectedIdentifier { actual } => actual.position(),
            AnalysisError::ExpectedIdentifierOrString { actual } => actual.position(),
            AnalysisError::ExpectedValueFromExpression { expression, .. } => expression.position(),
            AnalysisError::ExpectedValueArgumentCount { position, .. } => *position,
            AnalysisError::IndexOutOfBounds { index, .. } => index.position(),
            AnalysisError::LetExpectedValueFromStatement { actual } => actual.position(),
            AnalysisError::NegativeIndex { index, .. } => index.position(),
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
            AnalysisError::AstError(ast_error) => write!(f, "{}", ast_error),
            AnalysisError::ContextError { error, .. } => write!(f, "{}", error),
            AnalysisError::ExpectedType {
                expected,
                actual,
                actual_expression,
            } => {
                write!(
                    f,
                    "Expected type {}, found {} in {}",
                    expected, actual, actual_expression
                )
            }
            AnalysisError::ExpectedTypeMultiple {
                expected,
                actual,
                actual_expression,
            } => {
                write!(f, "Expected ")?;

                for (i, expected_type) in expected.iter().enumerate() {
                    if i == expected.len() - 1 {
                        write!(f, "or ")?;
                    } else if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}", expected_type)?;
                }

                write!(f, ", found {} in {}", actual, actual_expression)
            }

            AnalysisError::ExpectedIdentifier { actual, .. } => {
                write!(f, "Expected identifier, found {}", actual)
            }
            AnalysisError::ExpectedIdentifierOrString { actual } => {
                write!(f, "Expected identifier or string, found {}", actual)
            }
            AnalysisError::ExpectedValueFromExpression { expression } => {
                write!(f, "Expected {} to produce a value", expression)
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
            AnalysisError::LetExpectedValueFromStatement { actual, .. } => {
                write!(
                    f,
                    "Cannot assign to nothing. This expression should produce a value, but {} does not",
                    actual
                )
            }
            AnalysisError::NegativeIndex {
                list, index_value, ..
            } => write!(f, "Negative index {} for list {}", index_value, list),
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
                expression: map,
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
    fn multiple_errors() {
        let source = "1 + 1.0; 'a' + 1";

        assert_eq!(
            analyze(source),
            Err(DustError::Analysis {
                analysis_errors: vec![
                    AnalysisError::ExpectedType {
                        expected: Type::Integer,
                        actual: Type::Float,
                        actual_expression: Expression::literal(1.0, (4, 7)),
                    },
                    AnalysisError::ExpectedTypeMultiple {
                        expected: vec![Type::Float, Type::Integer, Type::String { length: None }],
                        actual: Type::Character,
                        actual_expression: Expression::literal('a', (9, 12)),
                    }
                ],
                source,
            })
        );
    }

    #[test]
    fn add_assign_wrong_type() {
        let source = "
            let mut a = 1;
            a += 1.0
        ";

        assert_eq!(
            analyze(source),
            Err(DustError::Analysis {
                analysis_errors: vec![AnalysisError::ExpectedType {
                    expected: Type::Integer,
                    actual: Type::Float,
                    actual_expression: Expression::literal(1.0, (45, 48)),
                }],
                source,
            })
        );
    }

    #[test]
    fn subtract_assign_wrong_type() {
        let source = "
            let mut a = 1;
            a -= 1.0
        ";

        assert_eq!(
            analyze(source),
            Err(DustError::Analysis {
                analysis_errors: vec![AnalysisError::ExpectedType {
                    expected: Type::Integer,
                    actual: Type::Float,
                    actual_expression: Expression::literal(1.0, (45, 48)),
                }],
                source,
            })
        );
    }

    #[test]
    fn tuple_struct_with_wrong_field_types() {
        let source = "
            struct Foo(int, float);
            Foo(1, 2)
        ";

        assert_eq!(
            analyze(source),
            Err(DustError::Analysis {
                analysis_errors: vec![AnalysisError::ExpectedType {
                    expected: Type::Float,
                    actual: Type::Integer,
                    actual_expression: Expression::literal(2, (52, 53)),
                }],
                source,
            })
        );
    }

    #[test]
    fn constant_list_index_out_of_bounds() {
        let source = "[1, 2, 3][3]";

        assert_eq!(
            analyze(source),
            Err(DustError::Analysis {
                analysis_errors: vec![AnalysisError::IndexOutOfBounds {
                    list: Expression::list(
                        vec![
                            Expression::literal(1, (1, 2)),
                            Expression::literal(2, (4, 5)),
                            Expression::literal(3, (7, 8)),
                        ],
                        (0, 9)
                    ),
                    index: Expression::literal(3, (10, 11)),
                    index_value: 3,
                    length: 3,
                }],
                source,
            })
        );
    }

    #[test]
    fn nonexistant_field_identifier() {
        let source = "{ x = 1 }.y";

        assert_eq!(
            analyze(source),
            Err(DustError::Analysis {
                analysis_errors: vec![AnalysisError::UndefinedField {
                    identifier: Expression::identifier(Identifier::new("y"), (11, 12)),
                    expression: Expression::map(
                        [(
                            Node::new(Identifier::new("x"), (2, 3)),
                            Expression::literal(1, (6, 7))
                        )],
                        (0, 11)
                    ),
                }],
                source,
            })
        );
    }

    #[test]
    fn nonexistant_field_string() {
        let source = "{ x = 1 }.'y'";

        assert_eq!(
            analyze(source),
            Err(DustError::Analysis {
                analysis_errors: vec![AnalysisError::UndefinedField {
                    identifier: Expression::literal("y", (11, 14)),
                    expression: Expression::map(
                        [(
                            Node::new(Identifier::new("x"), (2, 3)),
                            Expression::literal(1, (6, 7))
                        )],
                        (0, 11)
                    ),
                }],
                source,
            })
        );
    }

    #[test]
    fn malformed_list_index() {
        let source = "[1, 2, 3][\"foo\"]";

        assert_eq!(
            analyze(source),
            Err(DustError::Analysis {
                analysis_errors: vec![AnalysisError::ExpectedTypeMultiple {
                    expected: vec![
                        Type::Integer,
                        Type::Range {
                            r#type: RangeableType::Integer
                        }
                    ],
                    actual: Type::String { length: Some(3) },
                    actual_expression: Expression::literal("foo", (10, 15)),
                }],
                source,
            })
        );
    }

    #[test]
    fn malformed_field_access() {
        let source = "{ x = 1 }.0";

        assert_eq!(
            analyze(source),
            Err(DustError::Analysis {
                analysis_errors: vec![AnalysisError::ExpectedIdentifierOrString {
                    actual: Expression::literal(0, (10, 11))
                }],
                source,
            })
        );
    }

    #[test]
    fn float_plus_integer() {
        let source = "42.0 + 2";

        assert_eq!(
            analyze(source),
            Err(DustError::Analysis {
                analysis_errors: vec![AnalysisError::ExpectedType {
                    expected: Type::Float,
                    actual: Type::Integer,
                    actual_expression: Expression::literal(2, (7, 8)),
                }],
                source,
            })
        );
    }

    #[test]
    fn integer_plus_boolean() {
        let source = "42 + true";

        assert_eq!(
            analyze(source),
            Err(DustError::Analysis {
                analysis_errors: vec![AnalysisError::ExpectedType {
                    expected: Type::Integer,
                    actual: Type::Boolean,
                    actual_expression: Expression::literal(true, (5, 9)),
                }],
                source,
            })
        );
    }

    #[test]
    fn nonexistant_field() {
        let source = "'hello'.foo";

        assert_eq!(
            analyze(source),
            Err(DustError::Analysis {
                analysis_errors: vec![AnalysisError::UndefinedField {
                    expression: Expression::literal("hello", (0, 7)),
                    identifier: Expression::identifier(Identifier::new("foo"), (8, 11)),
                }],
                source,
            })
        );
    }

    #[test]
    fn undefined_variable() {
        let source = "foo";

        assert_eq!(
            analyze(source),
            Err(DustError::Analysis {
                analysis_errors: vec![AnalysisError::UndefinedVariable {
                    identifier: Node::new(Identifier::new("foo"), (0, 3))
                }],
                source,
            })
        );
    }
}
