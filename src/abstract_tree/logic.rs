use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractTree, Action, Expression, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Logic {
    Equal(WithPosition<Expression>, WithPosition<Expression>),
    NotEqual(WithPosition<Expression>, WithPosition<Expression>),
    Greater(WithPosition<Expression>, WithPosition<Expression>),
    Less(WithPosition<Expression>, WithPosition<Expression>),
    GreaterOrEqual(WithPosition<Expression>, WithPosition<Expression>),
    LessOrEqual(WithPosition<Expression>, WithPosition<Expression>),
    And(WithPosition<Expression>, WithPosition<Expression>),
    Or(WithPosition<Expression>, WithPosition<Expression>),
    Not(WithPosition<Expression>),
}

impl AbstractTree for Logic {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::Boolean)
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        match self {
            Logic::Equal(left, right)
            | Logic::NotEqual(left, right)
            | Logic::Greater(left, right)
            | Logic::Less(left, right)
            | Logic::GreaterOrEqual(left, right)
            | Logic::LessOrEqual(left, right) => {
                let left_type = left.node.expected_type(context)?;
                let right_type = right.node.expected_type(context)?;

                left_type
                    .check(&right_type)
                    .map_err(|conflict| ValidationError::TypeCheck {
                        conflict,
                        actual_position: left.position,
                        expected_position: right.position,
                    })?;

                Ok(())
            }
            Logic::And(left, right) | Logic::Or(left, right) => {
                let left_type = left.node.expected_type(context)?;
                let right_type = right.node.expected_type(context)?;

                if let Type::Boolean = left_type {
                } else {
                    return Err(ValidationError::ExpectedBoolean {
                        actual: left_type,
                        position: left.position,
                    });
                }

                if let Type::Boolean = right_type {
                } else {
                    return Err(ValidationError::ExpectedBoolean {
                        actual: right_type,
                        position: right.position,
                    });
                }

                Ok(())
            }
            Logic::Not(expression) => {
                let expression_type = expression.node.expected_type(context)?;

                if let Type::Boolean = expression_type {
                    Ok(())
                } else {
                    Err(ValidationError::ExpectedBoolean {
                        actual: expression_type,
                        position: expression.position,
                    })
                }
            }
        }
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let boolean = match self {
            Logic::Equal(left, right) => {
                let left = left.node.run(_context)?.as_return_value()?;
                let right = right.node.run(_context)?.as_return_value()?;

                left == right
            }
            Logic::NotEqual(left, right) => {
                let left = left.node.run(_context)?.as_return_value()?;
                let right = right.node.run(_context)?.as_return_value()?;

                left != right
            }
            Logic::Greater(left, right) => {
                let left = left.node.run(_context)?.as_return_value()?;
                let right = right.node.run(_context)?.as_return_value()?;

                left > right
            }
            Logic::Less(left, right) => {
                let left = left.node.run(_context)?.as_return_value()?;
                let right = right.node.run(_context)?.as_return_value()?;

                left < right
            }
            Logic::GreaterOrEqual(left, right) => {
                let left = left.node.run(_context)?.as_return_value()?;
                let right = right.node.run(_context)?.as_return_value()?;

                left >= right
            }
            Logic::LessOrEqual(left, right) => {
                let left = left.node.run(_context)?.as_return_value()?;
                let right = right.node.run(_context)?.as_return_value()?;

                left <= right
            }
            Logic::And(left, right) => {
                let left_value = left.node.run(_context)?.as_return_value()?;
                let right_value = right.node.run(_context)?.as_return_value()?;

                let left = if let ValueInner::Boolean(boolean) = left_value.inner().as_ref() {
                    boolean
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedBoolean {
                            actual: left_value.r#type(),
                            position: left.position,
                        },
                    ));
                };
                let right = if let ValueInner::Boolean(boolean) = right_value.inner().as_ref() {
                    boolean
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedBoolean {
                            actual: right_value.r#type(),
                            position: right.position,
                        },
                    ));
                };

                *left && *right
            }
            Logic::Or(left, right) => {
                let left_value = left.node.run(_context)?.as_return_value()?;
                let right_value = right.node.run(_context)?.as_return_value()?;

                let left = if let ValueInner::Boolean(boolean) = left_value.inner().as_ref() {
                    boolean
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedBoolean {
                            actual: left_value.r#type(),
                            position: left.position,
                        },
                    ));
                };
                let right = if let ValueInner::Boolean(boolean) = right_value.inner().as_ref() {
                    boolean
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedBoolean {
                            actual: right_value.r#type(),
                            position: right.position,
                        },
                    ));
                };

                *left || *right
            }
            Logic::Not(statement) => {
                let value = statement.node.run(_context)?.as_return_value()?;

                if let ValueInner::Boolean(boolean) = value.inner().as_ref() {
                    !boolean
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedBoolean {
                            actual: value.r#type(),
                            position: statement.position,
                        },
                    ));
                }
            }
        };

        Ok(Action::Return(Value::boolean(boolean)))
    }
}

#[cfg(test)]
mod tests {
    use crate::abstract_tree::ValueNode;

    use super::*;

    #[test]
    fn equal() {
        assert!(Logic::Equal(
            Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
            Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
        )
        .run(&Context::new())
        .unwrap()
        .as_return_value()
        .unwrap()
        .as_boolean()
        .unwrap())
    }

    #[test]
    fn not_equal() {
        assert!(Logic::NotEqual(
            Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
            Expression::Value(ValueNode::Integer(43)).with_position((0, 0)),
        )
        .run(&Context::new())
        .unwrap()
        .as_return_value()
        .unwrap()
        .as_boolean()
        .unwrap())
    }

    #[test]
    fn greater() {
        assert!(Logic::Greater(
            Expression::Value(ValueNode::Integer(43)).with_position((0, 0)),
            Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
        )
        .run(&Context::new())
        .unwrap()
        .as_return_value()
        .unwrap()
        .as_boolean()
        .unwrap())
    }

    #[test]
    fn less() {
        assert!(Logic::Less(
            Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
            Expression::Value(ValueNode::Integer(43)).with_position((0, 0)),
        )
        .run(&Context::new())
        .unwrap()
        .as_return_value()
        .unwrap()
        .as_boolean()
        .unwrap())
    }

    #[test]
    fn greater_or_equal() {
        assert!(Logic::GreaterOrEqual(
            Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
            Expression::Value(ValueNode::Integer(41)).with_position((0, 0)),
        )
        .run(&Context::new())
        .unwrap()
        .as_return_value()
        .unwrap()
        .as_boolean()
        .unwrap());

        assert!(Logic::GreaterOrEqual(
            Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
            Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
        )
        .run(&Context::new())
        .unwrap()
        .as_return_value()
        .unwrap()
        .as_boolean()
        .unwrap())
    }

    #[test]
    fn less_or_equal() {
        assert!(Logic::LessOrEqual(
            Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
            Expression::Value(ValueNode::Integer(43)).with_position((0, 0)),
        )
        .run(&Context::new())
        .unwrap()
        .as_return_value()
        .unwrap()
        .as_boolean()
        .unwrap());

        assert!(Logic::LessOrEqual(
            Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
            Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
        )
        .run(&Context::new())
        .unwrap()
        .as_return_value()
        .unwrap()
        .as_boolean()
        .unwrap())
    }

    #[test]
    fn and() {
        assert!(Logic::And(
            Expression::Value(ValueNode::Boolean(true)).with_position((0, 0)),
            Expression::Value(ValueNode::Boolean(true)).with_position((0, 0)),
        )
        .run(&Context::new())
        .unwrap()
        .as_return_value()
        .unwrap()
        .as_boolean()
        .unwrap())
    }

    #[test]
    fn or() {
        assert!(Logic::Or(
            Expression::Value(ValueNode::Boolean(true)).with_position((0, 0)),
            Expression::Value(ValueNode::Boolean(false)).with_position((0, 0)),
        )
        .run(&Context::new())
        .unwrap()
        .as_return_value()
        .unwrap()
        .as_boolean()
        .unwrap())
    }

    #[test]
    fn not() {
        assert!(
            Logic::Not(Expression::Value(ValueNode::Boolean(false)).with_position((0, 0)))
                .run(&Context::new())
                .unwrap()
                .as_return_value()
                .unwrap()
                .as_boolean()
                .unwrap()
        )
    }
}
