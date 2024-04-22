use crate::{
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
    value::ValueInner,
    Context, Value,
};

use super::{AbstractNode, Action, Statement, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Assignment {
    identifier: WithPosition<Identifier>,
    r#type: Option<WithPosition<Type>>,
    operator: AssignmentOperator,
    statement: Box<Statement>,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubAssign,
}

impl Assignment {
    pub fn new(
        identifier: WithPosition<Identifier>,
        r#type: Option<WithPosition<Type>>,
        operator: AssignmentOperator,
        statement: Statement,
    ) -> Self {
        Self {
            identifier,
            r#type,
            operator,
            statement: Box::new(statement),
        }
    }
}

impl AbstractNode for Assignment {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        let statement_type = self.statement.expected_type(context)?;

        if let Some(WithPosition {
            node: expected_type,
            position: expected_position,
        }) = &self.r#type
        {
            expected_type.check(&statement_type).map_err(|conflict| {
                ValidationError::TypeCheck {
                    conflict,
                    actual_position: self.statement.position(),
                    expected_position: expected_position.clone(),
                }
            })?;

            context.set_type(self.identifier.node.clone(), expected_type.clone())?;
        } else {
            context.set_type(self.identifier.node.clone(), statement_type)?;
        }

        self.statement.validate(context)?;

        Ok(())
    }

    fn run(self, context: &mut Context, clear_variables: bool) -> Result<Action, RuntimeError> {
        let action = self.statement.run(context, clear_variables)?;
        let right = match action {
            Action::Return(value) => value,
            r#break => return Ok(r#break),
        };

        match self.operator {
            AssignmentOperator::Assign => {
                context.set_value(self.identifier.node, right)?;
            }
            AssignmentOperator::AddAssign => {
                if let Some(left) = context.use_value(&self.identifier.node)? {
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
                if let Some(left) = context.use_value(&self.identifier.node)? {
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

        Ok(Action::None)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        abstract_tree::{Expression, ValueNode, WithPos},
        error::TypeConflict,
    };

    use super::*;

    #[test]
    fn assign_value() {
        let mut context = Context::new();

        Assignment::new(
            Identifier::new("foobar").with_position((0, 0)),
            None,
            AssignmentOperator::Assign,
            Statement::Expression(Expression::Value(
                ValueNode::Integer(42).with_position((0, 0)),
            )),
        )
        .run(&mut context, true)
        .unwrap();

        assert_eq!(
            context.use_value(&Identifier::new("foobar")),
            Ok(Some(Value::integer(42)))
        )
    }

    #[test]
    fn add_assign_value() {
        let mut context = Context::new();

        context
            .set_value(Identifier::new("foobar"), Value::integer(1))
            .unwrap();

        Assignment::new(
            Identifier::new("foobar").with_position((0, 0)),
            None,
            AssignmentOperator::AddAssign,
            Statement::Expression(Expression::Value(
                ValueNode::Integer(41).with_position((0, 0)),
            )),
        )
        .run(&mut context, true)
        .unwrap();

        assert_eq!(
            context.use_value(&Identifier::new("foobar")),
            Ok(Some(Value::integer(42)))
        )
    }

    #[test]
    fn subtract_assign_value() {
        let mut context = Context::new();

        context
            .set_value(Identifier::new("foobar"), Value::integer(43))
            .unwrap();

        Assignment::new(
            Identifier::new("foobar").with_position((0, 0)),
            None,
            AssignmentOperator::SubAssign,
            Statement::Expression(Expression::Value(
                ValueNode::Integer(1).with_position((0, 0)),
            )),
        )
        .run(&mut context, true)
        .unwrap();

        assert_eq!(
            context.use_value(&Identifier::new("foobar")),
            Ok(Some(Value::integer(42)))
        )
    }

    #[test]
    fn type_check() {
        let validation = Assignment::new(
            Identifier::new("foobar").with_position((0, 0)),
            Some(WithPosition {
                node: Type::Boolean,
                position: (0, 0).into(),
            }),
            AssignmentOperator::Assign,
            Statement::Expression(Expression::Value(
                ValueNode::Integer(42).with_position((0, 0)),
            )),
        )
        .validate(&Context::new());

        assert_eq!(
            validation,
            Err(ValidationError::TypeCheck {
                conflict: TypeConflict {
                    actual: Type::Integer,
                    expected: Type::Boolean
                },
                actual_position: (0, 0).into(),
                expected_position: (0, 0).into(),
            })
        )
    }
}
