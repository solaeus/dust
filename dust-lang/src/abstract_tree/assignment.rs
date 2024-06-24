use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
    value::ValueInner,
    Context, Value,
};

use super::{AbstractNode, Evaluation, Expression, Statement, Type, TypeConstructor, WithPosition};

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

impl AbstractNode for Assignment {
    fn define_types(&self, context: &Context) -> Result<(), ValidationError> {
        self.statement.define_types(context)?;

        if let Some(constructor) = &self.constructor {
            let r#type = constructor.construct(&context)?;

            context.set_type(self.identifier.node.clone(), r#type.clone())?;
        } else if let Some(r#type) = self.statement.expected_type(context)? {
            context.set_type(self.identifier.node.clone(), r#type)?;
        } else {
            return Err(ValidationError::CannotAssignToNone(
                self.statement.position(),
            ));
        };

        let relevant_statement = self.statement.last_evaluated_statement();

        if let (Some(constructor), Statement::Expression(Expression::FunctionCall(function_call))) =
            (&self.constructor, relevant_statement)
        {
            let declared_type = constructor.clone().construct(context)?;
            let function_type = function_call
                .node
                .function_expression()
                .expected_type(context)?;

            if let Some(Type::Function {
                return_type,
                type_parameters: Some(type_parameters),
                ..
            }) = function_type
            {
                if let Some(Type::Generic { identifier, .. }) = return_type.map(|r#box| *r#box) {
                    let returned_parameter = type_parameters
                        .into_iter()
                        .find(|parameter| parameter == &identifier);

                    if let Some(parameter) = returned_parameter {
                        context.set_type(parameter, declared_type)?;

                        return Ok(());
                    }
                }
            } else {
                return Err(ValidationError::ExpectedFunction {
                    actual: function_type.unwrap(),
                    position: function_call.position,
                });
            }
        }

        Ok(())
    }

    fn validate(&self, context: &Context, manage_memory: bool) -> Result<(), ValidationError> {
        self.statement.validate(context, manage_memory)
    }

    fn evaluate(
        self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let evaluation = self.statement.evaluate(context, manage_memory)?;
        let right = match evaluation {
            Some(Evaluation::Return(value)) => value,
            evaluation => return Ok(evaluation),
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

        Ok(None)
    }

    fn expected_type(&self, _: &Context) -> Result<Option<Type>, ValidationError> {
        Ok(None)
    }
}
