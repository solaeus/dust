use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
    value::ValueInner,
    Context, Value,
};

use super::{AbstractNode, Evaluation, ExpectedType, Statement, TypeConstructor, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Assignment {
    identifier: WithPosition<Identifier>,
    constructor: Option<WithPosition<TypeConstructor>>,
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
        constructor: Option<WithPosition<TypeConstructor>>,
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
    fn validate(&self, context: &mut Context, manage_memory: bool) -> Result<(), ValidationError> {
        let statement_type = self.statement.expected_type(context)?;

        if let Some(WithPosition {
            node: constructor,
            position: expected_position,
        }) = &self.constructor
        {
            let r#type = constructor.clone().construct(&context)?;

            r#type
                .check(&statement_type)
                .map_err(|conflict| ValidationError::TypeCheck {
                    conflict,
                    actual_position: self.statement.position(),
                    expected_position: Some(expected_position.clone()),
                })?;

            context.set_type(self.identifier.node.clone(), r#type.clone())?;
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
        let action = self.statement.evaluate(context, manage_memory)?;
        let right = match action {
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
