use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractNode, Action, Expression, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Logic {
    Equal(Expression, Expression),
    NotEqual(Expression, Expression),
    Greater(Expression, Expression),
    Less(Expression, Expression),
    GreaterOrEqual(Expression, Expression),
    LessOrEqual(Expression, Expression),
    And(Expression, Expression),
    Or(Expression, Expression),
    Not(Expression),
}

impl AbstractNode for Logic {
    fn expected_type(&self, _context: &mut Context) -> Result<Type, ValidationError> {
        Ok(Type::Boolean)
    }

    fn validate(&self, context: &mut Context, _manage_memory: bool) -> Result<(), ValidationError> {
        match self {
            Logic::Equal(left, right)
            | Logic::NotEqual(left, right)
            | Logic::Greater(left, right)
            | Logic::Less(left, right)
            | Logic::GreaterOrEqual(left, right)
            | Logic::LessOrEqual(left, right) => {
                left.validate(context, _manage_memory)?;
                right.validate(context, _manage_memory)?;

                let left_type = left.expected_type(context)?;
                let right_type = right.expected_type(context)?;

                left_type
                    .check(&right_type)
                    .map_err(|conflict| ValidationError::TypeCheck {
                        conflict,
                        actual_position: left.position(),
                        expected_position: right.position(),
                    })?;

                Ok(())
            }
            Logic::And(left, right) | Logic::Or(left, right) => {
                left.validate(context, _manage_memory)?;
                right.validate(context, _manage_memory)?;

                let left_type = left.expected_type(context)?;
                let right_type = right.expected_type(context)?;

                if let Type::Boolean = left_type {
                } else {
                    return Err(ValidationError::ExpectedBoolean {
                        actual: left_type,
                        position: left.position(),
                    });
                }

                if let Type::Boolean = right_type {
                } else {
                    return Err(ValidationError::ExpectedBoolean {
                        actual: right_type,
                        position: right.position(),
                    });
                }

                Ok(())
            }
            Logic::Not(expression) => {
                expression.validate(context, _manage_memory)?;

                let expression_type = expression.expected_type(context)?;

                if let Type::Boolean = expression_type {
                    Ok(())
                } else {
                    Err(ValidationError::ExpectedBoolean {
                        actual: expression_type,
                        position: expression.position(),
                    })
                }
            }
        }
    }

    fn run(self, context: &mut Context, _manage_memory: bool) -> Result<Action, RuntimeError> {
        let run_and_expect_value = |expression: Expression| -> Result<Value, RuntimeError> {
            let expression_position = expression.position();
            let action = expression.run(&mut context.clone(), _manage_memory)?;
            let value = if let Action::Return(value) = action {
                value
            } else {
                return Err(RuntimeError::ValidationFailure(
                    ValidationError::InterpreterExpectedReturn(expression_position),
                ));
            };

            Ok(value)
        };

        let run_and_expect_boolean = |expression: Expression| -> Result<bool, RuntimeError> {
            let expression_position = expression.position();
            let action = expression.run(&mut context.clone(), _manage_memory)?;
            let value = if let Action::Return(value) = action {
                value
            } else {
                return Err(RuntimeError::ValidationFailure(
                    ValidationError::InterpreterExpectedReturn(expression_position),
                ));
            };

            if let ValueInner::Boolean(boolean) = value.inner().as_ref() {
                Ok(*boolean)
            } else {
                return Err(RuntimeError::ValidationFailure(
                    ValidationError::ExpectedBoolean {
                        actual: value.r#type(context)?,
                        position: expression_position,
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
    use crate::abstract_tree::{ValueNode, WithPos};

    use super::*;

    #[test]
    fn equal() {
        assert_eq!(
            Logic::Equal(
                Expression::Value(ValueNode::Integer(42).with_position((0, 0))),
                Expression::Value(ValueNode::Integer(42).with_position((0, 0)))
            )
            .run(&mut Context::new(None), true),
            Ok(Action::Return(Value::boolean(true)))
        )
    }

    #[test]
    fn not_equal() {
        assert_eq!(
            Logic::NotEqual(
                Expression::Value(ValueNode::Integer(42).with_position((0, 0))),
                Expression::Value(ValueNode::Integer(43).with_position((0, 0)))
            )
            .run(&mut Context::new(None), true),
            Ok(Action::Return(Value::boolean(true)))
        )
    }

    #[test]
    fn greater() {
        assert_eq!(
            Logic::Greater(
                Expression::Value(ValueNode::Integer(43).with_position((0, 0))),
                Expression::Value(ValueNode::Integer(42).with_position((0, 0)))
            )
            .run(&mut Context::new(None), true),
            Ok(Action::Return(Value::boolean(true)))
        )
    }

    #[test]
    fn less() {
        assert_eq!(
            Logic::Less(
                Expression::Value(ValueNode::Integer(42).with_position((0, 0))),
                Expression::Value(ValueNode::Integer(43).with_position((0, 0)))
            )
            .run(&mut Context::new(None), true),
            Ok(Action::Return(Value::boolean(true)))
        )
    }

    #[test]
    fn greater_or_equal() {
        assert_eq!(
            Logic::GreaterOrEqual(
                Expression::Value(ValueNode::Integer(42).with_position((0, 0))),
                Expression::Value(ValueNode::Integer(41).with_position((0, 0)))
            )
            .run(&mut Context::new(None), true),
            Ok(Action::Return(Value::boolean(true)))
        );

        assert_eq!(
            Logic::GreaterOrEqual(
                Expression::Value(ValueNode::Integer(42).with_position((0, 0))),
                Expression::Value(ValueNode::Integer(42).with_position((0, 0))),
            )
            .run(&mut Context::new(None), true),
            Ok(Action::Return(Value::boolean(true)))
        );
    }

    #[test]
    fn less_or_equal() {
        assert_eq!(
            Logic::LessOrEqual(
                Expression::Value(ValueNode::Integer(41).with_position((0, 0))),
                Expression::Value(ValueNode::Integer(42).with_position((0, 0))),
            )
            .run(&mut Context::new(None), true),
            Ok(Action::Return(Value::boolean(true)))
        );

        assert_eq!(
            Logic::LessOrEqual(
                Expression::Value(ValueNode::Integer(42).with_position((0, 0))),
                Expression::Value(ValueNode::Integer(42).with_position((0, 0))),
            )
            .run(&mut Context::new(None), true),
            Ok(Action::Return(Value::boolean(true)))
        );
    }

    #[test]
    fn and() {
        assert_eq!(
            Logic::And(
                Expression::Value(ValueNode::Boolean(true).with_position((0, 0))),
                Expression::Value(ValueNode::Boolean(true).with_position((0, 0))),
            )
            .run(&mut Context::new(None), true),
            Ok(Action::Return(Value::boolean(true)))
        )
    }

    #[test]
    fn or() {
        assert_eq!(
            Logic::Or(
                Expression::Value(ValueNode::Boolean(true).with_position((0, 0))),
                Expression::Value(ValueNode::Boolean(false).with_position((0, 0))),
            )
            .run(&mut Context::new(None), true),
            Ok(Action::Return(Value::boolean(true)))
        )
    }

    #[test]
    fn not() {
        assert_eq!(
            Logic::Not(Expression::Value(
                ValueNode::Boolean(false).with_position((0, 0))
            ))
            .run(&mut Context::new(None), true),
            Ok(Action::Return(Value::boolean(true)))
        )
    }
}
