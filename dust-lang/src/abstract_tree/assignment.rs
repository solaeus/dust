use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
    value::ValueInner,
    Context, Value,
};

use super::{
    Evaluate, Evaluation, ExpectedType, Expression, Statement, Type, TypeConstructor, WithPosition,
};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Assignment {
    identifier: WithPosition<Identifier>,
    constructor: Option<TypeConstructor>,
    operator: AssignmentOperator,
    statement: Box<Statement>,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubAssign,
}

impl Assignment {
    pub fn new(
        identifier: WithPosition<Identifier>,
        constructor: Option<TypeConstructor>,
        operator: AssignmentOperator,
        statement: Statement,
    ) -> Self {
        Self {
            identifier,
            constructor,
            operator,
            statement: Box::new(statement),
        }
    }
}

impl Evaluate for Assignment {
    fn validate(&self, context: &mut Context, manage_memory: bool) -> Result<(), ValidationError> {
        let statement_type = self.statement.expected_type(context)?;

        if let Type::None = statement_type {
            return Err(ValidationError::CannotAssignToNone(
                self.statement.position(),
            ));
        }

        let statement = self
            .statement
            .last_child_statement()
            .unwrap_or(&self.statement);

        if let (Some(constructor), Statement::Expression(Expression::FunctionCall(function_call))) =
            (&self.constructor, statement)
        {
            let declared_type = constructor.clone().construct(context)?;
            let function_type = function_call.node.function().expected_type(context)?;

            if let Type::Function {
                return_type,
                type_parameters: Some(type_parameters),
                ..
            } = function_type
            {
                if let Type::Generic { identifier, .. } = *return_type {
                    let returned_parameter = type_parameters
                        .into_iter()
                        .find(|parameter| parameter == &identifier);

                    if let Some(parameter) = returned_parameter {
                        context.set_type(parameter, declared_type)?;
                    }
                }
            } else {
                return Err(ValidationError::ExpectedFunction {
                    actual: function_type,
                    position: function_call.position,
                });
            }
        } else {
            context.set_type(self.identifier.node.clone(), statement_type)?;
        }

        self.statement.validate(context, manage_memory)?;

        Ok(())
    }

    fn evaluate(
        self,
        context: &mut Context,
        manage_memory: bool,
    ) -> Result<Evaluation, RuntimeError> {
        let evaluation = self.statement.evaluate(context, manage_memory)?;
        let right = match evaluation {
            Evaluation::Return(value) => value,
            r#break => return Ok(r#break),
        };

        match self.operator {
            AssignmentOperator::Assign => {
                context.set_value(self.identifier.node, right)?;
            }
            AssignmentOperator::AddAssign => {
                let left_option = if manage_memory {
                    context.use_value(&self.identifier.node)?
                } else {
                    context.get_value(&self.identifier.node)?
                };

                if let Some(left) = left_option {
                    let new_value = match (left.inner().as_ref(), right.inner().as_ref()) {
                        (ValueInner::Integer(left), ValueInner::Integer(right)) => {
                            let sum = left.saturating_add(*right);

                            Value::integer(sum)
                        }
                        (ValueInner::Float(left), ValueInner::Float(right)) => {
                            let sum = left + right;

                            Value::float(sum)
                        }
                        (ValueInner::Float(left), ValueInner::Integer(right)) => {
                            let sum = left + *right as f64;

                            Value::float(sum)
                        }
                        (ValueInner::Integer(left), ValueInner::Float(right)) => {
                            let sum = *left as f64 + right;

                            Value::float(sum)
                        }
                        _ => {
                            return Err(RuntimeError::ValidationFailure(
                                ValidationError::ExpectedIntegerOrFloat(self.identifier.position),
                            ))
                        }
                    };
                    context.set_value(self.identifier.node, new_value)?;
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::VariableNotFound {
                            identifier: self.identifier.node,
                            position: self.identifier.position,
                        },
                    ));
                }
            }
            AssignmentOperator::SubAssign => {
                let left_option = if manage_memory {
                    context.use_value(&self.identifier.node)?
                } else {
                    context.get_value(&self.identifier.node)?
                };

                if let Some(left) = left_option {
                    let new_value = match (left.inner().as_ref(), right.inner().as_ref()) {
                        (ValueInner::Integer(left), ValueInner::Integer(right)) => {
                            let difference = left.saturating_sub(*right);

                            Value::integer(difference)
                        }
                        (ValueInner::Float(left), ValueInner::Float(right)) => {
                            let difference = left - right;

                            Value::float(difference)
                        }
                        (ValueInner::Float(left), ValueInner::Integer(right)) => {
                            let difference = left - *right as f64;

                            Value::float(difference)
                        }
                        (ValueInner::Integer(left), ValueInner::Float(right)) => {
                            let difference = *left as f64 - right;

                            Value::float(difference)
                        }
                        _ => {
                            return Err(RuntimeError::ValidationFailure(
                                ValidationError::ExpectedIntegerOrFloat(self.identifier.position),
                            ))
                        }
                    };
                    context.set_value(self.identifier.node, new_value)?;
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::VariableNotFound {
                            identifier: self.identifier.node,
                            position: self.identifier.position,
                        },
                    ));
                }
            }
        }

        Ok(Evaluation::None)
    }
}
