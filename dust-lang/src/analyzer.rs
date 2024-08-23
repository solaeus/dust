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
    StructType, Type, TypeConflict, TypeEvaluation,
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
/// let context = Context::new();
/// let mut analyzer = Analyzer::new(&abstract_tree, context);
/// let result = analyzer.analyze();
///
/// assert!(!analyzer.errors.is_empty());
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
            Statement::Expression(expression) => {
                self.analyze_expression(expression, statement.position())
            }
            Statement::ExpressionNullified(expression_node) => {
                self.analyze_expression(&expression_node.inner, statement.position());
            }
            Statement::Let(let_statement) => match &let_statement.inner {
                LetStatement::Let { identifier, value }
                | LetStatement::LetMut { identifier, value } => {
                    let r#type = match value.type_evaluation(&self.context) {
                        Err(ast_error) => {
                            self.errors.push(AnalysisError::AstError(ast_error));

                            return;
                        }
                        Ok(TypeEvaluation::Constructor(StructType::Unit { name })) => {
                            let set_type = self.context.set_variable_type(
                                identifier.inner.clone(),
                                Type::Struct(StructType::Unit { name }),
                                statement.position(),
                            );

                            if let Err(context_error) = set_type {
                                self.errors.push(AnalysisError::ContextError {
                                    error: context_error,
                                    position: identifier.position,
                                });
                            }

                            self.analyze_expression(value, statement.position());

                            return;
                        }
                        Ok(evaluation) => evaluation.r#type(),
                    };

                    if let Some(r#type) = r#type {
                        let set_type = self.context.set_variable_type(
                            identifier.inner.clone(),
                            r#type.clone(),
                            statement.position(),
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

                    self.analyze_expression(value, statement.position());
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
                        statement.position(),
                    ),
                    StructDefinition::Tuple { name, items } => {
                        let fields = items.iter().map(|item| item.inner.clone()).collect();

                        self.context.set_constructor_type(
                            name.inner.clone(),
                            StructType::Tuple {
                                name: name.inner.clone(),
                                fields,
                            },
                            statement.position(),
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
                            statement.position(),
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

    fn analyze_expression(&mut self, expression: &Expression, statement_position: Span) {
        match expression {
            Expression::Block(block_expression) => self.analyze_block(&block_expression.inner),
            Expression::Break(break_node) => {
                if let Some(expression) = &break_node.inner {
                    self.analyze_expression(expression, statement_position);
                }
            }
            Expression::Call(call_expression) => {
                let CallExpression { invoker, arguments } = call_expression.inner.as_ref();

                self.analyze_expression(invoker, statement_position);

                let invoker_evaluation = match invoker.type_evaluation(&self.context) {
                    Ok(evaluation) => evaluation,
                    Err(ast_error) => {
                        self.errors.push(AnalysisError::AstError(ast_error));

                        return;
                    }
                };

                if let TypeEvaluation::Constructor(StructType::Tuple { fields, .. }) =
                    invoker_evaluation
                {
                    for (expected_type, argument) in fields.iter().zip(arguments.iter()) {
                        let actual_type = match argument.type_evaluation(&self.context) {
                            Ok(evaluation) => evaluation.r#type(),
                            Err(ast_error) => {
                                self.errors.push(AnalysisError::AstError(ast_error));

                                return;
                            }
                        };

                        if let Some(r#type) = actual_type {
                            let check = expected_type.check(&r#type);

                            if let Err(type_conflict) = check {
                                self.errors.push(AnalysisError::TypeConflict {
                                    actual_expression: argument.clone(),
                                    type_conflict,
                                });
                            }
                        }
                    }

                    return;
                }

                let invoked_type = if let Some(r#type) = invoker_evaluation.r#type() {
                    r#type
                } else {
                    self.errors
                        .push(AnalysisError::ExpectedValueFromExpression {
                            expression: invoker.clone(),
                        });

                    return;
                };
                let function_type = if let Type::Function(function_type) = invoked_type {
                    function_type
                } else {
                    self.errors.push(AnalysisError::ExpectedFunction {
                        actual: invoked_type,
                        actual_expression: invoker.clone(),
                    });

                    return;
                };

                let value_parameters =
                    if let Some(value_parameters) = &function_type.value_parameters {
                        value_parameters
                    } else {
                        if !arguments.is_empty() {
                            self.errors.push(AnalysisError::ExpectedValueArgumentCount {
                                expected: 0,
                                actual: arguments.len(),
                                position: invoker.position(),
                            });
                        }

                        return;
                    };

                for ((_, expected_type), argument) in value_parameters.iter().zip(arguments) {
                    self.analyze_expression(argument, statement_position);

                    let argument_evaluation = match argument.type_evaluation(&self.context) {
                        Ok(evaluation) => evaluation,
                        Err(error) => {
                            self.errors.push(AnalysisError::AstError(error));

                            continue;
                        }
                    };

                    let actual_type = if let Some(r#type) = argument_evaluation.r#type() {
                        r#type
                    } else {
                        self.errors
                            .push(AnalysisError::ExpectedValueFromExpression {
                                expression: argument.clone(),
                            });

                        continue;
                    };

                    if let Err(type_conflict) = expected_type.check(&actual_type) {
                        self.errors.push(AnalysisError::TypeConflict {
                            type_conflict,
                            actual_expression: argument.clone(),
                        });
                    }
                }

                for argument in arguments {
                    self.analyze_expression(argument, statement_position);
                }
            }
            Expression::FieldAccess(field_access_expression) => {
                let FieldAccessExpression { container, field } =
                    field_access_expression.inner.as_ref();

                let evaluation = match container.type_evaluation(&self.context) {
                    Ok(evaluation) => evaluation,
                    Err(ast_error) => {
                        self.errors.push(AnalysisError::AstError(ast_error));

                        return;
                    }
                };
                let container_type = match evaluation.r#type() {
                    Some(r#type) => r#type,
                    None => {
                        self.errors
                            .push(AnalysisError::ExpectedValueFromExpression {
                                expression: container.clone(),
                            });

                        return;
                    }
                };

                if !container_type.has_field(&field.inner) {
                    self.errors.push(AnalysisError::UndefinedFieldIdentifier {
                        identifier: field.clone(),
                        container: container.clone(),
                    });
                }

                self.analyze_expression(container, statement_position);
            }
            Expression::Grouped(expression) => {
                self.analyze_expression(expression.inner.as_ref(), statement_position);
            }
            Expression::Identifier(identifier) => {
                let find_identifier = self
                    .context
                    .update_last_position(&identifier.inner, statement_position);

                if let Ok(false) = find_identifier {
                    self.errors.push(AnalysisError::UndefinedVariable {
                        identifier: identifier.clone(),
                    });
                }

                if let Err(context_error) = find_identifier {
                    self.errors.push(AnalysisError::ContextError {
                        error: context_error,
                        position: identifier.position,
                    });
                }
            }
            Expression::If(if_expression) => {
                self.analyze_if(&if_expression.inner, statement_position)
            }
            Expression::List(list_expression) => match list_expression.inner.as_ref() {
                ListExpression::AutoFill {
                    repeat_operand,
                    length_operand,
                } => {
                    self.analyze_expression(repeat_operand, statement_position);
                    self.analyze_expression(length_operand, statement_position);
                }
                ListExpression::Ordered(expressions) => {
                    for expression in expressions {
                        self.analyze_expression(expression, statement_position);
                    }
                }
            },
            Expression::ListIndex(list_index_expression) => {
                let ListIndexExpression { list, index } = list_index_expression.inner.as_ref();

                self.analyze_expression(list, statement_position);
                self.analyze_expression(index, statement_position);

                let list_type_evaluation = match list.type_evaluation(&self.context) {
                    Ok(evaluation) => evaluation,
                    Err(ast_error) => {
                        self.errors.push(AnalysisError::AstError(ast_error));

                        return;
                    }
                };
                let list_type = match list_type_evaluation.r#type() {
                    Some(r#type) => r#type,
                    None => {
                        self.errors
                            .push(AnalysisError::ExpectedValueFromExpression {
                                expression: list.clone(),
                            });

                        return;
                    }
                };
                let index_type_evaluation = match index.type_evaluation(&self.context) {
                    Ok(evaluation) => evaluation,
                    Err(ast_error) => {
                        self.errors.push(AnalysisError::AstError(ast_error));

                        return;
                    }
                };
                let index_type = match index_type_evaluation.r#type() {
                    Some(r#type) => r#type,
                    None => {
                        self.errors
                            .push(AnalysisError::ExpectedValueFromExpression {
                                expression: list.clone(),
                            });

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
                            self.errors.push(AnalysisError::ListIndexOutOfBounds {
                                index: index.clone(),
                                length,
                                list: list.clone(),
                                index_value: integer,
                            });
                        }
                    } else if let Type::Integer
                    | Type::Range {
                        r#type: RangeableType::Integer,
                    } = index_type
                    {
                    } else {
                        self.errors.push(AnalysisError::ExpectedTypeMultiple {
                            expected: vec![
                                Type::Integer,
                                Type::Range {
                                    r#type: RangeableType::Integer,
                                },
                            ],
                            actual: index_type.clone(),
                            actual_expression: index.clone(),
                        });
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
                            self.errors.push(AnalysisError::ListIndexOutOfBounds {
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
                    self.analyze_expression(condition, statement_position);
                    self.analyze_block(&block.inner);
                }
                LoopExpression::For {
                    iterator, block, ..
                } => {
                    self.analyze_expression(iterator, statement_position);
                    self.analyze_block(&block.inner);
                }
            },
            Expression::Map(map_expression) => {
                let MapExpression { pairs } = map_expression.inner.as_ref();

                for (_, expression) in pairs {
                    self.analyze_expression(expression, statement_position);
                }
            }
            Expression::Operator(operator_expression) => match operator_expression.inner.as_ref() {
                OperatorExpression::Assignment { assignee, value } => {
                    self.analyze_expression(assignee, statement_position);
                    self.analyze_expression(value, statement_position);
                }
                OperatorExpression::Comparison { left, right, .. } => {
                    self.analyze_expression(left, statement_position);
                    self.analyze_expression(right, statement_position);
                }
                OperatorExpression::CompoundAssignment {
                    assignee, modifier, ..
                } => {
                    self.analyze_expression(assignee, statement_position);
                    self.analyze_expression(modifier, statement_position);

                    let assignee_type_evaluation = match assignee.type_evaluation(&self.context) {
                        Ok(evaluation) => evaluation,
                        Err(ast_error) => {
                            self.errors.push(AnalysisError::AstError(ast_error));

                            return;
                        }
                    };
                    let modifier_type_evaluation = match modifier.type_evaluation(&self.context) {
                        Ok(evaluation) => evaluation,
                        Err(ast_error) => {
                            self.errors.push(AnalysisError::AstError(ast_error));

                            return;
                        }
                    };

                    let (expected_type, actual_type) = match (
                        assignee_type_evaluation.r#type(),
                        modifier_type_evaluation.r#type(),
                    ) {
                        (Some(expected_type), Some(actual_type)) => (expected_type, actual_type),
                        (None, None) => {
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
                        (None, _) => {
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: assignee.clone(),
                                });
                            return;
                        }
                        (_, None) => {
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: modifier.clone(),
                                });
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
                    self.analyze_expression(expression, statement_position);
                }
                OperatorExpression::Not(expression) => {
                    self.analyze_expression(expression, statement_position);
                }
                OperatorExpression::Math { left, right, .. } => {
                    self.analyze_expression(left, statement_position);
                    self.analyze_expression(right, statement_position);

                    let left_type_evaluation = match left.type_evaluation(&self.context) {
                        Ok(evaluation) => evaluation,
                        Err(ast_error) => {
                            self.errors.push(AnalysisError::AstError(ast_error));

                            return;
                        }
                    };
                    let right_type_evaluation = match right.type_evaluation(&self.context) {
                        Ok(evaluation) => evaluation,
                        Err(ast_error) => {
                            self.errors.push(AnalysisError::AstError(ast_error));

                            return;
                        }
                    };

                    let (left_type, right_type) = match (
                        left_type_evaluation.r#type(),
                        right_type_evaluation.r#type(),
                    ) {
                        (Some(left_type), Some(right_type)) => (left_type, right_type),
                        (None, None) => {
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
                        (None, _) => {
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: left.clone(),
                                });
                            return;
                        }
                        (_, None) => {
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: right.clone(),
                                });
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
                    self.analyze_expression(left, statement_position);
                    self.analyze_expression(right, statement_position);

                    let left_type_evaluation = match left.type_evaluation(&self.context) {
                        Ok(evaluation) => evaluation,
                        Err(ast_error) => {
                            self.errors.push(AnalysisError::AstError(ast_error));

                            return;
                        }
                    };
                    let right_type_evaluation = match right.type_evaluation(&self.context) {
                        Ok(evaluation) => evaluation,
                        Err(ast_error) => {
                            self.errors.push(AnalysisError::AstError(ast_error));

                            return;
                        }
                    };

                    let (left_type, right_type) = match (
                        left_type_evaluation.r#type(),
                        right_type_evaluation.r#type(),
                    ) {
                        (Some(left_type), Some(right_type)) => (left_type, right_type),
                        (None, None) => {
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
                        (None, _) => {
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: left.clone(),
                                });
                            return;
                        }
                        (_, None) => {
                            self.errors
                                .push(AnalysisError::ExpectedValueFromExpression {
                                    expression: right.clone(),
                                });
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
                    self.analyze_expression(start, statement_position);
                    self.analyze_expression(end, statement_position);
                }
                RangeExpression::Inclusive { start, end } => {
                    self.analyze_expression(start, statement_position);
                    self.analyze_expression(end, statement_position);
                }
            },
            Expression::Struct(struct_expression) => match struct_expression.inner.as_ref() {
                StructExpression::Fields { name, fields } => {
                    let update_position = self
                        .context
                        .update_last_position(&name.inner, statement_position);

                    if let Err(error) = update_position {
                        self.errors.push(AnalysisError::ContextError {
                            error,
                            position: name.position,
                        });

                        return;
                    }

                    for (_, expression) in fields {
                        self.analyze_expression(expression, statement_position);
                    }
                }
            },
            Expression::TupleAccess(tuple_access) => {
                let TupleAccessExpression { tuple, index } = tuple_access.inner.as_ref();

                let type_evaluation = match tuple.type_evaluation(&self.context) {
                    Ok(evaluation) => evaluation,
                    Err(ast_error) => {
                        self.errors.push(AnalysisError::AstError(ast_error));
                        return;
                    }
                };

                let tuple_type = match type_evaluation.r#type() {
                    Some(tuple_type) => tuple_type,
                    None => {
                        self.errors
                            .push(AnalysisError::ExpectedValueFromExpression {
                                expression: tuple.clone(),
                            });
                        return;
                    }
                };

                if let Type::Tuple {
                    fields: Some(fields),
                } = tuple_type
                {
                    if index.inner >= fields.len() {
                        self.errors.push(AnalysisError::TupleIndexOutOfBounds {
                            index: expression.clone(),
                            tuple: tuple.clone(),
                            index_value: index.inner as i64,
                            length: fields.len(),
                        });
                    }
                } else {
                    self.errors.push(AnalysisError::ExpectedType {
                        expected: Type::Tuple { fields: None },
                        actual: tuple_type,
                        actual_expression: tuple.clone(),
                    });
                }

                self.analyze_expression(tuple, statement_position);
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

    fn analyze_if(&mut self, if_expression: &IfExpression, statement_position: Span) {
        match if_expression {
            IfExpression::If {
                condition,
                if_block,
            } => {
                self.analyze_expression(condition, statement_position);
                self.analyze_block(&if_block.inner);
            }
            IfExpression::IfElse {
                condition,
                if_block,
                r#else,
            } => {
                self.analyze_expression(condition, statement_position);
                self.analyze_block(&if_block.inner);

                match r#else {
                    ElseExpression::Block(block_expression) => {
                        self.analyze_block(&block_expression.inner);
                    }
                    ElseExpression::If(if_expression) => {
                        self.analyze_if(&if_expression.inner, statement_position);
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
    ExpectedFunction {
        actual: Type,
        actual_expression: Expression,
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
    ListIndexOutOfBounds {
        list: Expression,
        index: Expression,
        index_value: i64,
        length: usize,
    },
    TupleIndexOutOfBounds {
        tuple: Expression,
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
        type_conflict: TypeConflict,
    },
    UnexpectedArguments {
        expected: Option<Vec<Type>>,
        actual: Vec<Expression>,
    },
    UndefinedFieldIdentifier {
        identifier: Node<Identifier>,
        container: Expression,
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
            AnalysisError::ExpectedFunction {
                actual_expression, ..
            } => actual_expression.position(),
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
            AnalysisError::ListIndexOutOfBounds { index, .. } => index.position(),
            AnalysisError::TupleIndexOutOfBounds { index, .. } => index.position(),
            AnalysisError::LetExpectedValueFromStatement { actual } => actual.position(),
            AnalysisError::NegativeIndex { index, .. } => index.position(),
            AnalysisError::TypeConflict {
                actual_expression, ..
            } => actual_expression.position(),
            AnalysisError::UndefinedFieldIdentifier { identifier, .. } => identifier.position,
            AnalysisError::UndefinedType { identifier } => identifier.position,
            AnalysisError::UndefinedVariable { identifier } => identifier.position,
            AnalysisError::UnexpectedArguments { actual, .. } => actual[0].position(),
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
            AnalysisError::ExpectedFunction {
                actual,
                actual_expression,
            } => {
                write!(
                    f,
                    "Expected function, found {} in {}",
                    actual, actual_expression
                )
            }
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
            AnalysisError::ListIndexOutOfBounds {
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
            AnalysisError::TupleIndexOutOfBounds {
                tuple,
                index_value,
                length,
                ..
            } => write!(
                f,
                "Index {} out of bounds for tuple {} with length {}",
                index_value, tuple, length
            ),
            AnalysisError::TypeConflict {
                actual_expression: actual_statement,
                type_conflict: TypeConflict { expected, actual },
            } => {
                write!(
                    f,
                    "Expected type {}, found {}, which has type {}",
                    expected, actual_statement, actual
                )
            }
            AnalysisError::UnexpectedArguments {
                actual, expected, ..
            } => {
                write!(
                    f,
                    "Unexpected arguments {:?}, expected {:?}",
                    actual, expected
                )
            }
            AnalysisError::UndefinedFieldIdentifier {
                identifier,
                container,
            } => {
                write!(
                    f,
                    "Undefined field {} in container {}",
                    identifier, container
                )
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
    use std::collections::HashMap;

    use crate::RangeableType;

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
                analysis_errors: vec![AnalysisError::TypeConflict {
                    actual_expression: Expression::literal(2, (56, 57)),
                    type_conflict: TypeConflict {
                        expected: Type::Float,
                        actual: Type::Integer,
                    },
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
                analysis_errors: vec![AnalysisError::ListIndexOutOfBounds {
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
    fn nonexistant_map_field_identifier() {
        let source = "map { x = 1 }.y";

        assert_eq!(
            analyze(source),
            Err(DustError::Analysis {
                analysis_errors: vec![AnalysisError::UndefinedFieldIdentifier {
                    container: Expression::map(
                        [(
                            Node::new(Identifier::new("x"), (6, 7)),
                            Expression::literal(1, (10, 11))
                        )],
                        (0, 13)
                    ),
                    identifier: Node::new(Identifier::new("y"), (14, 15)),
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
        let source = "struct Foo { x: int } Foo { x: 1 }.0";

        assert_eq!(
            analyze(source),
            Err(DustError::analysis(
                [AnalysisError::ExpectedType {
                    expected: Type::Tuple { fields: None },
                    actual: Type::Struct(StructType::Fields {
                        name: Identifier::new("Foo"),
                        fields: HashMap::from([(Identifier::new("x"), Type::Integer)])
                    }),
                    actual_expression: Expression::r#struct(
                        StructExpression::Fields {
                            name: Node::new(Identifier::new("Foo"), (22, 25)),
                            fields: vec![(
                                Node::new(Identifier::new("x"), (28, 29)),
                                Expression::literal(1, (31, 32))
                            )],
                        },
                        (22, 35)
                    ),
                }],
                source
            ))
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
        let source = "\"hello\".foo";

        assert_eq!(
            analyze(source),
            Err(DustError::Analysis {
                analysis_errors: vec![AnalysisError::UndefinedFieldIdentifier {
                    container: Expression::literal("hello", (0, 7)),
                    identifier: Node::new(Identifier::new("foo"), (8, 11)),
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
