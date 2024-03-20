use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractNode, Action, Expression, Type, WithPosition};

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

impl AbstractNode for Logic {
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

    fn run(self, context: &Context) -> Result<Action, RuntimeError> {
        let run_and_expect_value =
            |expression: WithPosition<Expression>| -> Result<Value, RuntimeError> {
                let action = expression.node.run(context)?;
                let value = if let Action::Return(value) = action {
                    value
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::InterpreterExpectedReturn(expression.position),
                    ));
                };

                Ok(value)
            };

        let run_and_expect_boolean =
            |expression: WithPosition<Expression>| -> Result<bool, RuntimeError> {
                let action = expression.node.run(context)?;
                let value = if let Action::Return(value) = action {
                    value
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::InterpreterExpectedReturn(expression.position),
                    ));
                };

                if let ValueInner::Boolean(boolean) = value.inner().as_ref() {
                    Ok(*boolean)
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedBoolean {
                            actual: value.r#type(context)?,
                            position: expression.position,
                        },
                    ));
                }
            };

        let boolean = match self {
            Logic::Equal(left, right) => {
                let (left_value, right_value) =
                    (run_and_expect_value(left)?, run_and_expect_value(right)?);

                left_value == right_value
            }
            Logic::NotEqual(left, right) => {
                let (left_value, right_value) =
                    (run_and_expect_value(left)?, run_and_expect_value(right)?);

                left_value != right_value
            }
            Logic::Greater(left, right) => {
                let (left_value, right_value) =
                    (run_and_expect_value(left)?, run_and_expect_value(right)?);

                left_value > right_value
            }
            Logic::Less(left, right) => {
                let (left_value, right_value) =
                    (run_and_expect_value(left)?, run_and_expect_value(right)?);

                left_value < right_value
            }
            Logic::GreaterOrEqual(left, right) => {
                let (left_value, right_value) =
                    (run_and_expect_value(left)?, run_and_expect_value(right)?);

                left_value >= right_value
            }
            Logic::LessOrEqual(left, right) => {
                let (left_value, right_value) =
                    (run_and_expect_value(left)?, run_and_expect_value(right)?);

                left_value <= right_value
            }
            Logic::And(left, right) => {
                let (left_boolean, right_boolean) = (
                    run_and_expect_boolean(left)?,
                    run_and_expect_boolean(right)?,
                );

                left_boolean && right_boolean
            }
            Logic::Or(left, right) => {
                let (left_boolean, right_boolean) = (
                    run_and_expect_boolean(left)?,
                    run_and_expect_boolean(right)?,
                );

                left_boolean || right_boolean
            }
            Logic::Not(statement) => {
                let boolean = run_and_expect_boolean(statement)?;

                !boolean
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
        assert_eq!(
            Logic::Equal(
                Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
                Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
            )
            .run(&Context::new()),
            Ok(Action::Return(Value::boolean(true)))
        )
    }

    #[test]
    fn not_equal() {
        assert_eq!(
            Logic::NotEqual(
                Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
                Expression::Value(ValueNode::Integer(43)).with_position((0, 0)),
            )
            .run(&Context::new()),
            Ok(Action::Return(Value::boolean(true)))
        )
    }

    #[test]
    fn greater() {
        assert_eq!(
            Logic::Greater(
                Expression::Value(ValueNode::Integer(43)).with_position((0, 0)),
                Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
            )
            .run(&Context::new()),
            Ok(Action::Return(Value::boolean(true)))
        )
    }

    #[test]
    fn less() {
        assert_eq!(
            Logic::Less(
                Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
                Expression::Value(ValueNode::Integer(43)).with_position((0, 0)),
            )
            .run(&Context::new()),
            Ok(Action::Return(Value::boolean(true)))
        )
    }

    #[test]
    fn greater_or_equal() {
        assert_eq!(
            Logic::GreaterOrEqual(
                Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
                Expression::Value(ValueNode::Integer(41)).with_position((0, 0)),
            )
            .run(&Context::new()),
            Ok(Action::Return(Value::boolean(true)))
        );

        assert_eq!(
            Logic::GreaterOrEqual(
                Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
                Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
            )
            .run(&Context::new()),
            Ok(Action::Return(Value::boolean(true)))
        );
    }

    #[test]
    fn less_or_equal() {
        assert_eq!(
            Logic::LessOrEqual(
                Expression::Value(ValueNode::Integer(41)).with_position((0, 0)),
                Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
            )
            .run(&Context::new()),
            Ok(Action::Return(Value::boolean(true)))
        );

        assert_eq!(
            Logic::LessOrEqual(
                Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
                Expression::Value(ValueNode::Integer(42)).with_position((0, 0)),
            )
            .run(&Context::new()),
            Ok(Action::Return(Value::boolean(true)))
        );
    }

    #[test]
    fn and() {
        assert_eq!(
            Logic::And(
                Expression::Value(ValueNode::Boolean(true)).with_position((0, 0)),
                Expression::Value(ValueNode::Boolean(true)).with_position((0, 0)),
            )
            .run(&Context::new()),
            Ok(Action::Return(Value::boolean(true)))
        )
    }

    #[test]
    fn or() {
        assert_eq!(
            Logic::Or(
                Expression::Value(ValueNode::Boolean(true)).with_position((0, 0)),
                Expression::Value(ValueNode::Boolean(false)).with_position((0, 0)),
            )
            .run(&Context::new()),
            Ok(Action::Return(Value::boolean(true)))
        )
    }

    #[test]
    fn not() {
        assert_eq!(
            Logic::Not(Expression::Value(ValueNode::Boolean(false)).with_position((0, 0)),)
                .run(&Context::new()),
            Ok(Action::Return(Value::boolean(true)))
        )
    }
}
