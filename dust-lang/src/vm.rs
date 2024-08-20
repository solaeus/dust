//! Virtual machine for running the abstract syntax tree.
//!
//! This module provides three running option:
//! - `run` convenience function that takes a source code string and runs it
//! - `run_with_context` convenience function that takes a source code string and a context
//! - `Vm` struct that can be used to run an abstract syntax tree
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    sync::{Arc, Mutex},
};

use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use crate::{
    ast::{
        AbstractSyntaxTree, BlockExpression, CallExpression, ComparisonOperator, ElseExpression,
        FieldAccessExpression, IfExpression, LetStatement, ListExpression, ListIndexExpression,
        LiteralExpression, LogicOperator, LoopExpression, MapExpression, MathOperator, Node,
        OperatorExpression, PrimitiveValueExpression, RangeExpression, Span, Statement,
        StructDefinition, StructExpression,
    },
    core_library, parse, Analyzer, BuiltInFunctionError, Constructor, Context, ContextData,
    ContextError, DustError, Expression, Function, FunctionCallError, Identifier, ParseError,
    StructType, Type, Value, ValueError,
};

/// Run the source code and return the result.
///
/// # Example
/// ```
/// # use dust_lang::vm::run;
/// # use dust_lang::value::Value;
/// let result = run("40 + 2");
///
/// assert_eq!(result, Ok(Some(Value::Integer(42))));
/// ```
pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let context = core_library().create_child();

    run_with_context(source, context)
}

/// Run the source code with a context and return the result.
///
/// # Example
/// ```
/// # use dust_lang::{Context, Identifier, Value, run_with_context};
/// let context = Context::new();
///
/// context.set_value(Identifier::new("foo"), Value::Integer(40));
/// context.update_last_position(&Identifier::new("foo"), (100, 100));
///
/// let result = run_with_context("foo + 2", context);
///
/// assert_eq!(result, Ok(Some(Value::Integer(42))));
/// ```
pub fn run_with_context(source: &str, context: Context) -> Result<Option<Value>, DustError> {
    let abstract_syntax_tree = parse(source)?;
    let analyzer = Analyzer::new(&abstract_syntax_tree, context.clone());

    analyzer
        .analyze()
        .map_err(|analysis_error| DustError::Analysis {
            analysis_error,
            source,
        })?;

    let mut vm = Vm::new(abstract_syntax_tree, context);

    vm.run().map_err(|runtime_error| DustError::Runtime {
        runtime_error,
        source,
    })
}

/// Dust virtual machine.
///
/// **Warning**: Do not run an AbstractSyntaxTree that has not been analyzed *with the same
/// context*. Use the `run` or `run_with_context` functions to make sure the program is analyzed
/// before running it.
///
/// See the `run_with_context` function for an example of how to use the Analyzer and the VM.
pub struct Vm {
    abstract_tree: AbstractSyntaxTree,
    context: Context,
}

impl Vm {
    pub fn new(abstract_tree: AbstractSyntaxTree, context: Context) -> Self {
        Self {
            abstract_tree,
            context,
        }
    }

    pub fn run(&mut self) -> Result<Option<Value>, RuntimeError> {
        let mut previous_evaluation = None;

        while let Some(statement) = self.abstract_tree.statements.pop_front() {
            previous_evaluation = self.run_statement(statement, true)?;
        }

        match previous_evaluation {
            Some(Evaluation::Break(value_option)) => Ok(value_option),
            Some(Evaluation::Return(value_option)) => Ok(value_option),
            _ => Ok(None),
        }
    }

    fn run_statement(
        &self,
        statement: Statement,
        collect_garbage: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        log::debug!("Running statement: {}", statement);

        let position = statement.position();
        let result = match statement {
            Statement::Expression(expression) => {
                Ok(Some(self.run_expression(expression, collect_garbage)?))
            }
            Statement::ExpressionNullified(expression) => {
                let evaluation = self.run_expression(expression.inner, collect_garbage)?;

                if let Evaluation::Break(_) = evaluation {
                    Ok(Some(evaluation))
                } else {
                    Ok(None)
                }
            }
            Statement::Let(let_statement) => {
                self.run_let_statement(let_statement.inner, collect_garbage)?;

                Ok(None)
            }
            Statement::StructDefinition(struct_definition) => {
                let (name, struct_type) = match struct_definition.inner {
                    StructDefinition::Unit { name } => {
                        (name.inner.clone(), StructType::Unit { name: name.inner })
                    }
                    StructDefinition::Tuple { name, items } => {
                        let fields = items.into_iter().map(|item| item.inner).collect();

                        (
                            name.inner.clone(),
                            StructType::Tuple {
                                name: name.inner,
                                fields,
                            },
                        )
                    }
                    StructDefinition::Fields { name, fields } => {
                        let fields = fields
                            .into_iter()
                            .map(|(identifier, r#type)| (identifier.inner, r#type.inner))
                            .collect();

                        (
                            name.inner.clone(),
                            StructType::Fields {
                                name: name.inner,
                                fields,
                            },
                        )
                    }
                };
                let constructor = struct_type.constructor();

                self.context
                    .set_constructor(name, constructor)
                    .map_err(|error| RuntimeError::ContextError {
                        error,
                        position: struct_definition.position,
                    })?;

                Ok(None)
            }
        };

        if collect_garbage {
            self.context
                .collect_garbage(position.1)
                .map_err(|error| RuntimeError::ContextError { error, position })?;
        }

        result.map_err(|error| RuntimeError::Statement {
            error: Box::new(error),
            position,
        })
    }

    fn run_let_statement(
        &self,
        let_statement: LetStatement,
        collect_garbage: bool,
    ) -> Result<(), RuntimeError> {
        match let_statement {
            LetStatement::Let { identifier, value } => {
                let position = value.position();
                let value = self
                    .run_expression(value, collect_garbage)?
                    .expect_value(position)?;

                self.context
                    .set_variable_value(identifier.inner, value)
                    .map_err(|error| RuntimeError::ContextError { error, position })?;

                Ok(())
            }
            LetStatement::LetMut { identifier, value } => {
                let position = value.position();
                let mutable_value = self
                    .run_expression(value, collect_garbage)?
                    .expect_value(position)?
                    .into_mutable();

                self.context
                    .set_variable_value(identifier.inner, mutable_value)
                    .map_err(|error| RuntimeError::ContextError { error, position })?;

                Ok(())
            }
            LetStatement::LetType { .. } => todo!(),
            LetStatement::LetMutType { .. } => todo!(),
        }
    }

    fn run_expression(
        &self,
        expression: Expression,
        collect_garbage: bool,
    ) -> Result<Evaluation, RuntimeError> {
        log::debug!("Running expression: {}", expression);

        let position = expression.position();
        let evaluation_result = match expression {
            Expression::Block(Node { inner, .. }) => self.run_block(*inner, collect_garbage),
            Expression::Break(Node { inner, .. }) => {
                let break_expression = if let Some(expression) = inner {
                    *expression
                } else {
                    return Ok(Evaluation::Break(None));
                };
                let run_break = self.run_expression(break_expression, collect_garbage)?;
                let evaluation = match run_break {
                    Evaluation::Break(value_option) => Evaluation::Break(value_option),
                    Evaluation::Return(value_option) => Evaluation::Break(value_option),
                    Evaluation::Constructor(_) => {
                        return Err(RuntimeError::ExpectedValue { position })
                    }
                };

                Ok(evaluation)
            }
            Expression::Call(call) => self.run_call(*call.inner, collect_garbage),
            Expression::FieldAccess(field_access) => {
                self.run_field_access(*field_access.inner, collect_garbage)
            }
            Expression::Grouped(expression) => {
                self.run_expression(*expression.inner, collect_garbage)
            }
            Expression::Identifier(identifier) => self.run_identifier(identifier),
            Expression::If(if_expression) => self.run_if(*if_expression.inner, collect_garbage),
            Expression::List(list_expression) => {
                self.run_list(*list_expression.inner, collect_garbage)
            }
            Expression::ListIndex(list_index) => {
                self.run_list_index(*list_index.inner, collect_garbage)
            }
            Expression::Literal(literal) => self.run_literal(*literal.inner),
            Expression::Loop(loop_expression) => self.run_loop(*loop_expression.inner),
            Expression::Map(map_expression) => self.run_map(*map_expression.inner, collect_garbage),
            Expression::Operator(operator_expression) => {
                self.run_operator(*operator_expression.inner, collect_garbage)
            }
            Expression::Range(range_expression) => {
                self.run_range(*range_expression.inner, collect_garbage)
            }
            Expression::Struct(struct_expression) => {
                self.run_struct(*struct_expression.inner, collect_garbage)
            }
            Expression::TupleAccess(_) => todo!(),
        };

        evaluation_result.map_err(|error| RuntimeError::Expression {
            error: Box::new(error),
            position,
        })
    }

    fn run_identifier(&self, identifier: Node<Identifier>) -> Result<Evaluation, RuntimeError> {
        log::debug!("Running identifier: {}", identifier);

        let get_data = self.context.get_data(&identifier.inner).map_err(|error| {
            RuntimeError::ContextError {
                error,
                position: identifier.position,
            }
        })?;

        if let Some(ContextData::VariableValue(value)) = get_data {
            return Ok(Evaluation::Return(Some(value)));
        }

        if let Some(ContextData::Constructor(constructor)) = get_data {
            if let Constructor::Unit(unit_constructor) = constructor {
                return Ok(Evaluation::Return(Some(unit_constructor.construct())));
            }

            return Ok(Evaluation::Constructor(constructor));
        }

        Err(RuntimeError::UnassociatedIdentifier {
            identifier: identifier.inner,
            position: identifier.position,
        })
    }

    fn run_struct(
        &self,
        struct_expression: StructExpression,
        collect_garbage: bool,
    ) -> Result<Evaluation, RuntimeError> {
        log::debug!("Running struct expression: {struct_expression}");

        let StructExpression::Fields { name, fields } = struct_expression;

        let position = name.position;
        let constructor = self
            .context
            .get_constructor(&name.inner)
            .map_err(|error| RuntimeError::ContextError { error, position })?;

        if let Some(constructor) = constructor {
            if let Constructor::Fields(fields_constructor) = constructor {
                let mut arguments = HashMap::with_capacity(fields.len());

                for (identifier, expression) in fields {
                    let position = expression.position();
                    let value = self
                        .run_expression(expression, collect_garbage)?
                        .expect_value(position)?;

                    arguments.insert(identifier.inner, value);
                }

                Ok(Evaluation::Return(Some(
                    fields_constructor.construct(arguments),
                )))
            } else {
                Err(RuntimeError::ExpectedFieldsConstructor { position })
            }
        } else {
            Err(RuntimeError::ExpectedConstructor { position })
        }
    }

    fn run_range(
        &self,
        range: RangeExpression,
        collect_garbage: bool,
    ) -> Result<Evaluation, RuntimeError> {
        match range {
            RangeExpression::Exclusive { start, end } => {
                let start_position = start.position();
                let start = self
                    .run_expression(start, collect_garbage)?
                    .expect_value(start_position)?;
                let end_position = end.position();
                let end = self
                    .run_expression(end, collect_garbage)?
                    .expect_value(end_position)?;

                match (start, end) {
                    (Value::Byte(start), Value::Byte(end)) => {
                        Ok(Evaluation::Return(Some(Value::range(start, end))))
                    }
                    (Value::Character(start), Value::Character(end)) => {
                        Ok(Evaluation::Return(Some(Value::range(start, end))))
                    }
                    (Value::Float(start), Value::Float(end)) => {
                        Ok(Evaluation::Return(Some(Value::range(start, end))))
                    }
                    (Value::Integer(start), Value::Integer(end)) => {
                        Ok(Evaluation::Return(Some(Value::range(start, end))))
                    }
                    _ => Err(RuntimeError::InvalidRange {
                        start_position,
                        end_position,
                    }),
                }
            }
            RangeExpression::Inclusive { start, end } => {
                let start_position = start.position();
                let start = self
                    .run_expression(start, collect_garbage)?
                    .expect_value(start_position)?;
                let end_position = end.position();
                let end = self
                    .run_expression(end, collect_garbage)?
                    .expect_value(end_position)?;

                match (start, end) {
                    (Value::Byte(start), Value::Byte(end)) => {
                        Ok(Evaluation::Return(Some(Value::range_inclusive(start, end))))
                    }
                    (Value::Character(start), Value::Character(end)) => {
                        Ok(Evaluation::Return(Some(Value::range_inclusive(start, end))))
                    }
                    (Value::Float(start), Value::Float(end)) => {
                        Ok(Evaluation::Return(Some(Value::range_inclusive(start, end))))
                    }
                    (Value::Integer(start), Value::Integer(end)) => {
                        Ok(Evaluation::Return(Some(Value::range_inclusive(start, end))))
                    }
                    _ => Err(RuntimeError::InvalidRange {
                        start_position,
                        end_position,
                    }),
                }
            }
        }
    }

    fn run_map(
        &self,
        map: MapExpression,
        collect_garbage: bool,
    ) -> Result<Evaluation, RuntimeError> {
        let MapExpression { pairs } = map;

        let mut map = HashMap::new();

        for (identifier, expression) in pairs {
            let expression_position = expression.position();
            let value = self
                .run_expression(expression, collect_garbage)?
                .expect_value(expression_position)?;

            map.insert(identifier.inner, value);
        }

        Ok(Evaluation::Return(Some(Value::Map(map))))
    }

    fn run_operator(
        &self,
        operator: OperatorExpression,
        collect_garbage: bool,
    ) -> Result<Evaluation, RuntimeError> {
        match operator {
            OperatorExpression::Assignment { assignee, value } => {
                let assignee_position = assignee.position();
                let assignee = self
                    .run_expression(assignee, collect_garbage)?
                    .expect_value(assignee_position)?;
                let value_position = value.position();
                let value = self
                    .run_expression(value, collect_garbage)?
                    .expect_value(value_position)?;

                assignee
                    .mutate(value)
                    .map_err(|error| RuntimeError::ValueError {
                        error,
                        left_position: assignee_position,
                        right_position: value_position,
                    })?;

                Ok(Evaluation::Return(None))
            }
            OperatorExpression::Comparison {
                left,
                operator,
                right,
            } => {
                let left_position = left.position();
                let left_value = self
                    .run_expression(left, collect_garbage)?
                    .expect_value(left_position)?;
                let right_position = right.position();
                let right_value = self
                    .run_expression(right, collect_garbage)?
                    .expect_value(right_position)?;
                let outcome =
                    match operator.inner {
                        ComparisonOperator::Equal => left_value.equal(&right_value),
                        ComparisonOperator::NotEqual => left_value.not_equal(&right_value),
                        ComparisonOperator::GreaterThan => left_value
                            .greater_than(&right_value)
                            .map_err(|error| RuntimeError::ValueError {
                                error,
                                left_position,
                                right_position,
                            })?,
                        ComparisonOperator::GreaterThanOrEqual => left_value
                            .greater_than_or_equal(&right_value)
                            .map_err(|error| RuntimeError::ValueError {
                                error,
                                left_position,
                                right_position,
                            })?,
                        ComparisonOperator::LessThan => left_value
                            .less_than(&right_value)
                            .map_err(|error| RuntimeError::ValueError {
                                error,
                                left_position,
                                right_position,
                            })?,
                        ComparisonOperator::LessThanOrEqual => left_value
                            .less_than_or_equal(&right_value)
                            .map_err(|error| RuntimeError::ValueError {
                                error,
                                left_position,
                                right_position,
                            })?,
                    };

                Ok(Evaluation::Return(Some(outcome)))
            }
            OperatorExpression::CompoundAssignment {
                assignee,
                operator,
                modifier,
            } => {
                let assignee_position = assignee.position();
                let assignee = self
                    .run_expression(assignee, collect_garbage)?
                    .expect_value(assignee_position)?;
                let modifier_position = modifier.position();
                let modifier = self
                    .run_expression(modifier, collect_garbage)?
                    .expect_value(modifier_position)?;

                match operator.inner {
                    MathOperator::Add => assignee.add_assign(&modifier),
                    MathOperator::Subtract => assignee.subtract_assign(&modifier),
                    MathOperator::Multiply => assignee.multiply_assign(&modifier),
                    MathOperator::Divide => assignee.divide_assign(&modifier),
                    MathOperator::Modulo => assignee.modulo_assign(&modifier),
                }
                .map_err(|error| RuntimeError::ValueError {
                    error,
                    left_position: assignee_position,
                    right_position: modifier_position,
                })?;

                Ok(Evaluation::Return(None))
            }
            OperatorExpression::ErrorPropagation(_) => todo!(),
            OperatorExpression::Negation(expression) => {
                let position = expression.position();
                let value = self
                    .run_expression(expression, collect_garbage)?
                    .expect_value(position)?;
                let integer = value
                    .as_integer()
                    .ok_or(RuntimeError::ExpectedBoolean { position })?;
                let negated = Value::Integer(-integer);

                Ok(Evaluation::Return(Some(negated)))
            }
            OperatorExpression::Not(expression) => {
                let position = expression.position();
                let value = self
                    .run_expression(expression, collect_garbage)?
                    .expect_value(position)?;
                let boolean = value
                    .as_boolean()
                    .ok_or(RuntimeError::ExpectedBoolean { position })?;
                let not = Value::Boolean(!boolean);

                Ok(Evaluation::Return(Some(not)))
            }
            OperatorExpression::Math {
                left,
                operator,
                right,
            } => {
                let left_position = left.position();
                let left_value = self
                    .run_expression(left, collect_garbage)?
                    .expect_value(left_position)?;
                let right_position = right.position();
                let right_value = self
                    .run_expression(right, collect_garbage)?
                    .expect_value(right_position)?;
                let outcome = match operator.inner {
                    MathOperator::Add => left_value.add(&right_value),
                    MathOperator::Subtract => left_value.subtract(&right_value),
                    MathOperator::Multiply => left_value.multiply(&right_value),
                    MathOperator::Divide => left_value.divide(&right_value),
                    MathOperator::Modulo => left_value.modulo(&right_value),
                }
                .map_err(|value_error| RuntimeError::ValueError {
                    error: value_error,
                    left_position,
                    right_position,
                })?;

                Ok(Evaluation::Return(Some(outcome)))
            }
            OperatorExpression::Logic {
                left,
                operator,
                right,
            } => {
                let left_position = left.position();
                let left_value = self
                    .run_expression(left, collect_garbage)?
                    .expect_value(left_position)?;
                let right_position = right.position();
                let right_value = self
                    .run_expression(right, collect_garbage)?
                    .expect_value(right_position)?;
                let outcome = match operator.inner {
                    LogicOperator::And => left_value.and(&right_value),
                    LogicOperator::Or => left_value.or(&right_value),
                }
                .map_err(|value_error| RuntimeError::ValueError {
                    error: value_error,
                    left_position,
                    right_position,
                })?;

                Ok(Evaluation::Return(Some(outcome)))
            }
        }
    }

    fn run_loop(&self, loop_expression: LoopExpression) -> Result<Evaluation, RuntimeError> {
        match loop_expression {
            LoopExpression::Infinite {
                block: Node { inner, .. },
            } => match inner {
                BlockExpression::Sync(statements) => 'outer: loop {
                    for statement in statements.clone() {
                        let evaluation = self.run_statement(statement, false)?;

                        if let Some(Evaluation::Break(value_option)) = evaluation {
                            break 'outer Ok(Evaluation::Return(value_option));
                        }
                    }
                },
                BlockExpression::Async(_) => todo!(),
            },
            LoopExpression::While { condition, block } => {
                while self
                    .run_expression(condition.clone(), false)?
                    .expect_value(condition.position())?
                    .as_boolean()
                    .ok_or_else(|| RuntimeError::ExpectedBoolean {
                        position: condition.position(),
                    })?
                {
                    self.run_block(block.inner.clone(), false)?;
                }

                Ok(Evaluation::Return(None))
            }
            LoopExpression::For { .. } => todo!(),
        }
    }

    fn run_literal(&self, literal: LiteralExpression) -> Result<Evaluation, RuntimeError> {
        let value = match literal {
            LiteralExpression::BuiltInFunction(built_in_function) => {
                Value::Function(Function::BuiltIn(built_in_function))
            }
            LiteralExpression::String(string) => Value::String(string),
            LiteralExpression::Primitive(primitive_expression) => match primitive_expression {
                PrimitiveValueExpression::Boolean(boolean) => Value::Boolean(boolean),
                PrimitiveValueExpression::Character(character) => Value::Character(character),
                PrimitiveValueExpression::Integer(integer) => Value::Integer(integer),
                PrimitiveValueExpression::Float(float) => Value::Float(float),
            },
        };

        Ok(Evaluation::Return(Some(value)))
    }

    fn run_list_index(
        &self,
        list_index: ListIndexExpression,
        collect_garbage: bool,
    ) -> Result<Evaluation, RuntimeError> {
        let ListIndexExpression { list, index } = list_index;

        let list_position = list.position();
        let list_value = self
            .run_expression(list, collect_garbage)?
            .expect_value(list_position)?;

        let index_position = index.position();
        let index_value = self
            .run_expression(index, collect_garbage)?
            .expect_value(index_position)?;

        let get_index =
            list_value
                .get_index(index_value)
                .map_err(|error| RuntimeError::ValueError {
                    error,
                    left_position: list_position,
                    right_position: index_position,
                })?;

        Ok(Evaluation::Return(get_index))
    }

    fn run_call(
        &self,
        call_expression: CallExpression,
        collect_garbage: bool,
    ) -> Result<Evaluation, RuntimeError> {
        log::debug!("Running call expression: {call_expression}");

        let CallExpression { invoker, arguments } = call_expression;
        let invoker_position = invoker.position();

        if let Expression::FieldAccess(field_access) = invoker {
            let FieldAccessExpression { container, field } = *field_access.inner;

            let container_position = container.position();
            let container_value = self
                .run_expression(container, collect_garbage)?
                .expect_value(container_position)?;

            let function = if let Some(value) = container_value.get_field(&field.inner) {
                if let Value::Function(function) = value {
                    function
                } else {
                    return Err(RuntimeError::ExpectedFunction {
                        actual: value,
                        position: container_position,
                    });
                }
            } else {
                return Err(RuntimeError::UndefinedField {
                    value: container_value,
                    value_position: container_position,
                    property: field.inner,
                    property_position: field.position,
                });
            };

            let mut value_arguments = vec![container_value];

            for argument in arguments {
                let position = argument.position();
                let value = self
                    .run_expression(argument, collect_garbage)?
                    .expect_value(position)?;

                value_arguments.push(value);
            }

            let context = Context::new();

            return function
                .call(None, Some(value_arguments), &context)
                .map(Evaluation::Return)
                .map_err(|error| RuntimeError::FunctionCall {
                    error,
                    position: invoker_position,
                });
        }

        let invoker_position = invoker.position();
        let run_invoker = self.run_expression(invoker, collect_garbage)?;

        match run_invoker {
            Evaluation::Constructor(constructor) => match constructor {
                Constructor::Unit(unit_constructor) => {
                    Ok(Evaluation::Return(Some(unit_constructor.construct())))
                }
                Constructor::Tuple(tuple_constructor) => {
                    let mut fields = Vec::new();

                    for argument in arguments {
                        let position = argument.position();

                        if let Some(value) = self.run_expression(argument, collect_garbage)?.value()
                        {
                            fields.push(value);
                        } else {
                            return Err(RuntimeError::ExpectedValue { position });
                        }
                    }

                    let tuple = tuple_constructor.construct(fields);

                    Ok(Evaluation::Return(Some(tuple)))
                }
                Constructor::Fields(_) => {
                    todo!("Return an error")
                }
            },
            Evaluation::Return(Some(value)) => {
                let function = if let Value::Function(function) = value {
                    function
                } else {
                    return Err(RuntimeError::ExpectedFunction {
                        actual: value.to_owned(),
                        position: invoker_position,
                    });
                };

                let mut value_arguments: Option<Vec<Value>> = None;

                for argument in arguments {
                    let position = argument.position();
                    let value = self
                        .run_expression(argument, collect_garbage)?
                        .expect_value(position)?;

                    if let Some(value_arguments) = &mut value_arguments {
                        value_arguments.push(value);
                    } else {
                        value_arguments = Some(vec![value]);
                    }
                }

                let context = Context::new();

                function
                    .call(None, value_arguments, &context)
                    .map(Evaluation::Return)
                    .map_err(|error| RuntimeError::FunctionCall {
                        error,
                        position: invoker_position,
                    })
            }
            _ => Err(RuntimeError::ExpectedValueOrConstructor {
                position: invoker_position,
            }),
        }
    }

    fn run_field_access(
        &self,
        field_access: FieldAccessExpression,
        collect_garbage: bool,
    ) -> Result<Evaluation, RuntimeError> {
        let FieldAccessExpression { container, field } = field_access;

        let container_position = container.position();
        let container_value =
            if let Some(value) = self.run_expression(container, collect_garbage)?.value() {
                value
            } else {
                return Err(RuntimeError::ExpectedValue {
                    position: container_position,
                });
            };

        Ok(Evaluation::Return(container_value.get_field(&field.inner)))
    }

    fn run_list(
        &self,
        list_expression: ListExpression,
        collect_garbage: bool,
    ) -> Result<Evaluation, RuntimeError> {
        match list_expression {
            ListExpression::AutoFill {
                repeat_operand,
                length_operand,
            } => {
                let position = length_operand.position();
                let length = self
                    .run_expression(length_operand, collect_garbage)?
                    .expect_value(position)?
                    .as_integer()
                    .ok_or(RuntimeError::ExpectedInteger { position })?;

                let position = repeat_operand.position();
                let value = self
                    .run_expression(repeat_operand, collect_garbage)?
                    .expect_value(position)?;

                Ok(Evaluation::Return(Some(Value::List(vec![
                    value;
                    length as usize
                ]))))
            }
            ListExpression::Ordered(expressions) => {
                let mut values = Vec::new();

                for expression in expressions {
                    let position = expression.position();
                    let value = self
                        .run_expression(expression, collect_garbage)?
                        .expect_value(position)?;

                    values.push(value);
                }

                Ok(Evaluation::Return(Some(Value::List(values))))
            }
        }
    }

    fn run_block(
        &self,
        block: BlockExpression,
        collect_garbage: bool,
    ) -> Result<Evaluation, RuntimeError> {
        match block {
            BlockExpression::Async(statements) => {
                let final_result = Arc::new(Mutex::new(None));
                let statements_length = statements.len();
                let error_option =
                    statements
                        .into_par_iter()
                        .enumerate()
                        .find_map_any(|(i, statement)| {
                            let evaluation_result = self.run_statement(statement, false);

                            match evaluation_result {
                                Ok(evaluation_option) => {
                                    if i == statements_length - 1 {
                                        let mut final_result = final_result.lock().unwrap();

                                        *final_result = evaluation_option;
                                    }

                                    None
                                }
                                Err(error) => Some(error),
                            }
                        });

                if let Some(error) = error_option {
                    Err(error)
                } else if let Some(evaluation) = final_result.lock().unwrap().take() {
                    Ok(evaluation)
                } else {
                    Ok(Evaluation::Return(None))
                }
            }
            BlockExpression::Sync(statements) => {
                let mut previous_evaluation = None;

                for statement in statements {
                    previous_evaluation = self.run_statement(statement, collect_garbage)?;

                    if let Some(Evaluation::Break(value_option)) = previous_evaluation {
                        return Ok(Evaluation::Break(value_option));
                    }
                }

                match previous_evaluation {
                    Some(evaluation) => Ok(evaluation),
                    None => Ok(Evaluation::Return(None)),
                }
            }
        }
    }

    fn run_if(
        &self,
        if_expression: IfExpression,
        collect_garbage: bool,
    ) -> Result<Evaluation, RuntimeError> {
        match if_expression {
            IfExpression::If {
                condition,
                if_block,
            } => {
                let position = condition.position();
                let boolean = self
                    .run_expression(condition, collect_garbage)?
                    .expect_value(position)?
                    .as_boolean()
                    .ok_or(RuntimeError::ExpectedBoolean { position })?;

                if boolean {
                    let evaluation = self.run_block(if_block.inner, collect_garbage)?;

                    if let Evaluation::Break(_) = evaluation {
                        return Ok(evaluation);
                    }
                }

                Ok(Evaluation::Return(None))
            }
            IfExpression::IfElse {
                condition,
                if_block,
                r#else,
            } => {
                let position = condition.position();
                let boolean = self
                    .run_expression(condition, collect_garbage)?
                    .expect_value(position)?
                    .as_boolean()
                    .ok_or(RuntimeError::ExpectedBoolean { position })?;

                if boolean {
                    self.run_block(if_block.inner, collect_garbage)
                } else {
                    match r#else {
                        ElseExpression::If(if_expression) => {
                            self.run_if(*if_expression.inner, collect_garbage)
                        }
                        ElseExpression::Block(block) => {
                            self.run_block(block.inner, collect_garbage)
                        }
                    }
                }
            }
        }
    }
}

enum Evaluation {
    Break(Option<Value>),
    Constructor(Constructor),
    Return(Option<Value>),
}

impl Evaluation {
    pub fn value(self) -> Option<Value> {
        match self {
            Evaluation::Return(value_option) => value_option,
            _ => None,
        }
    }

    pub fn expect_value(self, position: Span) -> Result<Value, RuntimeError> {
        if let Evaluation::Return(Some(value)) = self {
            Ok(value)
        } else {
            Err(RuntimeError::ExpectedValue { position })
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeError {
    ContextError {
        error: ContextError,
        position: Span,
    },
    FunctionCall {
        error: FunctionCallError,
        position: Span,
    },
    ParseError(ParseError),
    Expression {
        error: Box<RuntimeError>,
        position: Span,
    },
    Statement {
        error: Box<RuntimeError>,
        position: Span,
    },
    ValueError {
        error: ValueError,
        left_position: Span,
        right_position: Span,
    },

    // Anaylsis Failures
    // These should be prevented by running the analyzer before the VM
    BuiltInFunctionError {
        error: BuiltInFunctionError,
        position: Span,
    },
    EnumVariantNotFound {
        identifier: Identifier,
        position: Span,
    },
    ExpectedBoolean {
        position: Span,
    },
    ExpectedConstructor {
        position: Span,
    },
    ExpectedFieldsConstructor {
        position: Span,
    },
    ExpectedIdentifier {
        position: Span,
    },
    ExpectedIntegerOrRange {
        position: Span,
    },
    ExpectedIdentifierOrString {
        position: Span,
    },
    ExpectedInteger {
        position: Span,
    },
    ExpectedNumber {
        position: Span,
    },
    ExpectedMap {
        position: Span,
    },
    ExpectedType {
        expected: Type,
        actual: Type,
        position: Span,
    },
    ExpectedFunction {
        actual: Value,
        position: Span,
    },
    ExpectedList {
        position: Span,
    },
    ExpectedValue {
        position: Span,
    },
    ExpectedValueOrConstructor {
        position: Span,
    },
    InvalidRange {
        start_position: Span,
        end_position: Span,
    },
    UnassociatedIdentifier {
        identifier: Identifier,
        position: Span,
    },
    UndefinedType {
        identifier: Identifier,
        position: Span,
    },
    UndefinedField {
        value: Value,
        value_position: Span,
        property: Identifier,
        property_position: Span,
    },
}

impl RuntimeError {
    pub fn position(&self) -> Span {
        match self {
            Self::ContextError { position, .. } => *position,
            Self::BuiltInFunctionError { position, .. } => *position,
            Self::FunctionCall { position, .. } => *position,
            Self::UnassociatedIdentifier { position, .. } => *position,
            Self::ParseError(parse_error) => parse_error.position(),
            Self::Expression { position, .. } => *position,
            Self::Statement { position, .. } => *position,
            Self::ValueError {
                left_position,
                right_position,
                ..
            } => (left_position.0, right_position.1),
            Self::EnumVariantNotFound { position, .. } => *position,
            Self::ExpectedBoolean { position } => *position,
            Self::ExpectedConstructor { position, .. } => *position,
            Self::ExpectedFieldsConstructor { position } => *position,
            Self::ExpectedFunction { position, .. } => *position,
            Self::ExpectedIdentifier { position } => *position,
            Self::ExpectedIdentifierOrString { position } => *position,
            Self::ExpectedInteger { position } => *position,
            Self::ExpectedIntegerOrRange { position } => *position,
            Self::ExpectedList { position } => *position,
            Self::ExpectedMap { position } => *position,
            Self::ExpectedNumber { position } => *position,
            Self::ExpectedType { position, .. } => *position,
            Self::ExpectedValue { position } => *position,
            Self::ExpectedValueOrConstructor { position } => *position,
            Self::InvalidRange {
                start_position,
                end_position,
                ..
            } => (start_position.0, end_position.1),

            Self::UndefinedType { position, .. } => *position,
            Self::UndefinedField {
                property_position, ..
            } => *property_position,
        }
    }
}

impl From<ParseError> for RuntimeError {
    fn from(error: ParseError) -> Self {
        Self::ParseError(error)
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::ContextError { error, position } => {
                write!(f, "Context error at {:?}: {}", position, error)
            }
            Self::FunctionCall { error, position } => {
                write!(f, "Function call error at {:?}: {}", position, error)
            }
            Self::ParseError(parse_error) => write!(f, "{}", parse_error),
            Self::Expression { error, position } => {
                write!(
                    f,
                    "Error while running expression at {:?}: {}",
                    position, error
                )
            }
            Self::Statement { error, position } => {
                write!(
                    f,
                    "Error while running statement at {:?}: {}",
                    position, error
                )
            }
            Self::ValueError {
                error,
                left_position,
                right_position,
            } => {
                write!(
                    f,
                    "Value error with values at positions: {:?} and {:?} {}",
                    left_position, right_position, error
                )
            }
            Self::BuiltInFunctionError { error, .. } => {
                write!(f, "{}", error)
            }
            Self::EnumVariantNotFound {
                identifier,
                position,
            } => {
                write!(
                    f,
                    "Enum variant not found: {} at position: {:?}",
                    identifier, position
                )
            }
            Self::ExpectedBoolean { position } => {
                write!(f, "Expected a boolean at position: {:?}", position)
            }
            Self::ExpectedConstructor { position } => {
                write!(f, "Expected a constructor at position: {:?}", position)
            }
            Self::ExpectedFieldsConstructor { position } => {
                write!(
                    f,
                    "Expected a fields constructor at position: {:?}",
                    position
                )
            }
            Self::ExpectedFunction { actual, position } => {
                write!(
                    f,
                    "Expected a function, but got {} at position: {:?}",
                    actual, position
                )
            }
            Self::ExpectedIdentifier { position } => {
                write!(f, "Expected an identifier at position: {:?}", position)
            }
            Self::ExpectedIdentifierOrString { position } => {
                write!(
                    f,
                    "Expected an identifier or string at position: {:?}",
                    position
                )
            }
            Self::ExpectedIntegerOrRange { position } => {
                write!(
                    f,
                    "Expected an identifier, integer, or range at position: {:?}",
                    position
                )
            }
            Self::ExpectedInteger { position } => {
                write!(f, "Expected an integer at position: {:?}", position)
            }
            Self::ExpectedList { position } => {
                write!(f, "Expected a list at position: {:?}", position)
            }
            Self::ExpectedType {
                expected,
                actual,
                position,
            } => {
                write!(
                    f,
                    "Expected type {}, but got {} at position: {:?}",
                    expected, actual, position
                )
            }
            Self::ExpectedMap { position } => {
                write!(f, "Expected a map at position: {:?}", position)
            }
            Self::ExpectedNumber { position } => {
                write!(
                    f,
                    "Expected an integer or float at position: {:?}",
                    position
                )
            }
            Self::ExpectedValue { position } => {
                write!(f, "Expected a value at position: {:?}", position)
            }
            Self::ExpectedValueOrConstructor { position } => {
                write!(
                    f,
                    "Expected a value or constructor at position: {:?}",
                    position
                )
            }
            Self::InvalidRange {
                start_position,
                end_position,
            } => {
                write!(
                    f,
                    "Invalid range with start position: {:?} and end position: {:?}",
                    start_position, end_position
                )
            }
            Self::UnassociatedIdentifier { identifier, .. } => {
                write!(
                    f,
                    "Identifier \"{identifier}\" is not associated with a value or constructor"
                )
            }
            Self::UndefinedField {
                value, property, ..
            } => {
                write!(f, "Value {} does not have the property {}", value, property)
            }
            Self::UndefinedType {
                identifier,
                position,
            } => {
                write!(
                    f,
                    "Undefined type {} at position: {:?}",
                    identifier, position
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::Struct;

    use super::*;

    #[test]
    fn break_loop() {
        let input = "let mut x = 0; loop { x += 1; if x == 10 { break; } } x";

        assert_eq!(run(input), Ok(Some(Value::mutable(Value::Integer(10)))));
    }

    #[test]
    fn string_index() {
        let input = "'foo'[0]";

        assert_eq!(run(input), Ok(Some(Value::Character('f'))));
    }

    #[test]
    fn map_expression() {
        let input = "let x = map { foo = 42, bar = 4.2 }; x";

        assert_eq!(
            run(input),
            Ok(Some(Value::map([
                (Identifier::new("foo"), Value::Integer(42)),
                (Identifier::new("bar"), Value::Float(4.2))
            ])))
        );
    }

    #[test]
    fn async_block() {
        let input = "let mut x = 1; async { x += 1; x -= 1; } x";

        assert_eq!(run(input), Ok(Some(Value::mutable(Value::Integer(1)))));
    }

    #[test]
    fn define_and_instantiate_fields_struct() {
        let input = "struct Foo { bar: int, baz: float } Foo { bar: 42, baz: 4.0 }";

        assert_eq!(
            run(input),
            Ok(Some(Value::Struct(Struct::Fields {
                name: Identifier::new("Foo"),
                fields: HashMap::from([
                    (Identifier::new("bar"), Value::Integer(42)),
                    (Identifier::new("baz"), Value::Float(4.0))
                ])
            })))
        );
    }

    #[test]
    fn assign_tuple_struct_variable() {
        let input = "
            struct Foo(int);
            let x = Foo(42);
            x
        ";

        assert_eq!(
            run(input),
            Ok(Some(Value::Struct(Struct::Tuple {
                name: Identifier::new("Foo"),
                fields: vec![Value::Integer(42)]
            })))
        )
    }

    #[test]
    fn define_and_instantiate_tuple_struct() {
        let input = "struct Foo(int); Foo(42)";

        assert_eq!(
            run(input),
            Ok(Some(Value::Struct(Struct::Tuple {
                name: Identifier::new("Foo"),
                fields: vec![Value::Integer(42)]
            })))
        );
    }

    #[test]
    fn assign_unit_struct_variable() {
        let input = "
            struct Foo;
            let x = Foo;
            x
        ";

        assert_eq!(
            run(input),
            Ok(Some(Value::Struct(Struct::Unit {
                name: Identifier::new("Foo")
            })))
        )
    }

    #[test]
    fn define_and_instantiate_unit_struct() {
        let input = "struct Foo; Foo";

        assert_eq!(
            run(input),
            Ok(Some(Value::Struct(Struct::Unit {
                name: Identifier::new("Foo")
            })))
        );
    }

    #[test]
    fn list_index_nested() {
        let input = "[[1, 2], [42, 4], [5, 6]][1][0]";

        assert_eq!(run(input), Ok(Some(Value::Integer(42))));
    }

    #[test]
    fn list_index_range() {
        let input = "[1, 2, 3, 4, 5][1..3]";

        assert_eq!(
            run(input),
            Ok(Some(Value::List(vec![
                Value::Integer(2),
                Value::Integer(3)
            ])))
        );
    }

    #[test]
    fn range() {
        let input = "1..5";

        assert_eq!(run(input), Ok(Some(Value::range(1, 5))));
    }

    #[test]
    fn negate_expression() {
        let input = "let x = -42; -x";

        assert_eq!(run(input), Ok(Some(Value::Integer(42))));
    }

    #[test]
    fn not_expression() {
        let input = "!(1 == 2 || 3 == 4 || 5 == 6)";

        assert_eq!(run(input), Ok(Some(Value::Boolean(true))));
    }

    #[test]
    fn list_index() {
        let input = "[1, 42, 3][1]";

        assert_eq!(run(input), Ok(Some(Value::Integer(42))));
    }

    #[test]
    fn map_property_access() {
        let input = "map { a = 42 }.a";

        assert_eq!(run(input), Ok(Some(Value::Integer(42))));
    }

    #[test]
    fn built_in_function_dot_notation() {
        let input = "42.to_string()";

        assert_eq!(run(input), Ok(Some(Value::string("42"))));
    }

    #[test]
    fn to_string() {
        let input = "to_string(42)";

        assert_eq!(run(input), Ok(Some(Value::string("42"))));
    }

    #[test]
    fn r#if() {
        let input = "if true { 1 }";

        assert_eq!(run(input), Ok(None));
    }

    #[test]
    fn if_else() {
        let input = "let x = if false { 1 } else { 2 }; x";

        assert_eq!(run(input), Ok(Some(Value::Integer(2))));
    }

    #[test]
    fn if_else_if() {
        let input = "if false { 1 } else if true { 2 }";

        assert_eq!(run(input), Ok(None));
    }

    #[test]
    fn if_else_if_else() {
        let input = "if false { 1 } else if false { 2 } else { 3 }";

        assert_eq!(run(input), Ok(Some(Value::Integer(3))));
    }

    #[test]
    fn while_loop() {
        let input = "let mut x = 0; while x < 5 { x += 1 } x";

        assert_eq!(run(input), Ok(Some(Value::mutable_from(5))));
    }

    #[test]
    fn subtract_assign() {
        let input = "let mut x = 1; x -= 1; x";

        assert_eq!(run(input), Ok(Some(Value::mutable_from(0))));
    }

    #[test]
    fn add_assign() {
        let input = "let mut x = 1; x += 1; x";

        assert_eq!(run(input), Ok(Some(Value::mutable_from(2))));
    }

    #[test]
    fn and() {
        let input = "true && true";

        assert_eq!(run(input), Ok(Some(Value::Boolean(true))));
    }

    #[test]
    fn or() {
        let input = "true || false";

        assert_eq!(run(input), Ok(Some(Value::Boolean(true))));
    }

    #[test]
    fn integer_equal() {
        let input = "42 == 42";

        assert_eq!(run(input), Ok(Some(Value::Boolean(true))));
    }

    #[test]
    fn modulo() {
        let input = "42 % 2";

        assert_eq!(run(input), Ok(Some(Value::Integer(0))));
    }

    #[test]
    fn divide() {
        let input = "42 / 2";

        assert_eq!(run(input), Ok(Some(Value::Integer(21))));
    }

    #[test]
    fn less_than() {
        let input = "2 < 3";

        assert_eq!(run(input), Ok(Some(Value::Boolean(true))));
    }

    #[test]
    fn less_than_or_equal() {
        let input = "42 <= 42";

        assert_eq!(run(input), Ok(Some(Value::Boolean(true))));
    }

    #[test]
    fn greater_than() {
        let input = "2 > 3";

        assert_eq!(run(input), Ok(Some(Value::Boolean(false))));
    }

    #[test]
    fn greater_than_or_equal() {
        let input = "42 >= 42";

        assert_eq!(run(input), Ok(Some(Value::Boolean(true))));
    }

    #[test]
    fn integer_saturating_add() {
        let input = "9223372036854775807 + 1";

        assert_eq!(run(input), Ok(Some(Value::Integer(i64::MAX))));
    }

    #[test]
    fn integer_saturating_sub() {
        let input = "-9223372036854775808 - 1";

        assert_eq!(run(input), Ok(Some(Value::Integer(i64::MIN))));
    }

    #[test]
    fn multiply() {
        let input = "2 * 3";

        assert_eq!(run(input), Ok(Some(Value::Integer(6))));
    }

    #[test]
    fn boolean() {
        let input = "true";

        assert_eq!(run(input), Ok(Some(Value::Boolean(true))));
    }

    #[test]
    fn is_even() {
        let input = "42.is_even";

        assert_eq!(run(input), Ok(Some(Value::Boolean(true))));
    }

    #[test]
    fn is_odd() {
        let input = "42.is_odd";

        assert_eq!(run(input), Ok(Some(Value::Boolean(false))));
    }

    #[test]
    fn list_length() {
        let input = "[1, 2, 3].length";

        assert_eq!(run(input), Ok(Some(Value::Integer(3))));
    }

    #[test]
    fn string_length() {
        let input = "\"hello\".length";

        assert_eq!(run(input), Ok(Some(Value::Integer(5))));
    }

    #[test]
    fn map_length() {
        let input = "map { a = 42, b = 4.0 }.length";

        assert_eq!(run(input), Ok(Some(Value::Integer(2))));
    }

    #[test]
    fn add() {
        let input = "1 + 2";

        assert_eq!(run(input), Ok(Some(Value::Integer(3))));
    }

    #[test]
    fn add_multiple() {
        let input = "1 + 2 + 3";

        assert_eq!(run(input), Ok(Some(Value::Integer(6))));
    }
}
