use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
    value::ValueInner,
    Context, Value,
};

use super::{
    AbstractNode, Evaluation, SourcePosition, Statement, Type, TypeConstructor, WithPosition,
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

impl AbstractNode for Assignment {
    fn define_and_validate(
        &self,
        context: &Context,
        manage_memory: bool,
        scope: SourcePosition,
    ) -> Result<(), ValidationError> {
        let r#type = if let Some(constructor) = &self.constructor {
            constructor.construct(&context)?
        } else if let Some(r#type) = self.statement.expected_type(context)? {
            r#type
        } else {
            return Err(ValidationError::CannotAssignToNone(
                self.statement.last_evaluated_statement().position(),
            ));
        };

        context.set_type(self.identifier.node.clone(), r#type, scope)?;
        self.statement
            .define_and_validate(context, manage_memory, scope)?;

        let statement_type = self.statement.expected_type(context)?;

        if statement_type.is_none() {
            return Err(ValidationError::CannotAssignToNone(
                self.statement.last_evaluated_statement().position(),
            ));
        }

        if let (Some(expected_type_constructor), Some(actual_type)) =
            (&self.constructor, statement_type)
        {
            let expected_type = expected_type_constructor.construct(context)?;

            expected_type
                .check(&actual_type)
                .map_err(|conflict| ValidationError::TypeCheck {
                    conflict,
                    actual_position: self.statement.last_evaluated_statement().position(),
                    expected_position: Some(expected_type_constructor.position()),
                })?;
        }

        Ok(())
    }

    fn evaluate(
        self,
        context: &Context,
        manage_memory: bool,
        scope: SourcePosition,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let evaluation = self.statement.evaluate(context, manage_memory, scope)?;
        let right = match evaluation {
            Some(Evaluation::Return(value)) => value,
            evaluation => return Ok(evaluation),
        };

        match self.operator {
            AssignmentOperator::Assign => {
                context.set_value(self.identifier.node, right, scope)?;
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
                    context.set_value(self.identifier.node, new_value, scope)?;
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
                    context.set_value(self.identifier.node, new_value, scope)?;
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

impl Display for Assignment {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Assignment {
            identifier,
            constructor,
            operator,
            statement,
        } = self;
        write!(f, "{} ", identifier.node)?;

        if let Some(constructor) = constructor {
            write!(f, ": {constructor} ")?;
        }

        match operator {
            AssignmentOperator::Assign => write!(f, "="),
            AssignmentOperator::AddAssign => write!(f, "+="),
            AssignmentOperator::SubAssign => write!(f, "-="),
        }?;

        write!(f, " {statement}")
    }
}
