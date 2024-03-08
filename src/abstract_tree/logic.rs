use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    Value,
};

use super::{AbstractTree, Action, Expression, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
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
                let left = left.expected_type(context)?;
                let right = right.expected_type(context)?;

                left.check(&right)?;

                Ok(())
            }
            Logic::And(left, right) | Logic::Or(left, right) => {
                let left = left.expected_type(context)?;
                let right = right.expected_type(context)?;

                if let (Type::Boolean, Type::Boolean) = (left, right) {
                    Ok(())
                } else {
                    Err(ValidationError::ExpectedBoolean)
                }
            }
            Logic::Not(expression) => {
                if let Type::Boolean = expression.expected_type(context)? {
                    Ok(())
                } else {
                    Err(ValidationError::ExpectedBoolean)
                }
            }
        }
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let boolean = match self {
            Logic::Equal(left, right) => {
                let left = left.run(_context)?.as_return_value()?;
                let right = right.run(_context)?.as_return_value()?;

                left == right
            }
            Logic::NotEqual(left, right) => {
                let left = left.run(_context)?.as_return_value()?;
                let right = right.run(_context)?.as_return_value()?;

                left != right
            }
            Logic::Greater(left, right) => {
                let left = left.run(_context)?.as_return_value()?;
                let right = right.run(_context)?.as_return_value()?;

                left > right
            }
            Logic::Less(left, right) => {
                let left = left.run(_context)?.as_return_value()?;
                let right = right.run(_context)?.as_return_value()?;

                left < right
            }
            Logic::GreaterOrEqual(left, right) => {
                let left = left.run(_context)?.as_return_value()?;
                let right = right.run(_context)?.as_return_value()?;

                left >= right
            }
            Logic::LessOrEqual(left, right) => {
                let left = left.run(_context)?.as_return_value()?;
                let right = right.run(_context)?.as_return_value()?;

                left <= right
            }
            Logic::And(left, right) => {
                let left = left.run(_context)?.as_return_value()?.as_boolean()?;
                let right = right.run(_context)?.as_return_value()?.as_boolean()?;

                left && right
            }
            Logic::Or(left, right) => {
                let left = left.run(_context)?.as_return_value()?.as_boolean()?;
                let right = right.run(_context)?.as_return_value()?.as_boolean()?;

                left || right
            }
            Logic::Not(statement) => {
                let boolean = statement.run(_context)?.as_return_value()?.as_boolean()?;

                !boolean
            }
        };

        Ok(Action::Return(Value::boolean(boolean)))
    }
}

#[cfg(test)]
mod tests {
    use crate::abstract_tree::{Expression, ValueNode};

    use super::*;

    #[test]
    fn equal() {
        assert!(Logic::Equal(
            Expression::Value(ValueNode::Integer(42)),
            Expression::Value(ValueNode::Integer(42)),
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
            Expression::Value(ValueNode::Integer(42)),
            Expression::Value(ValueNode::Integer(43)),
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
            Expression::Value(ValueNode::Integer(43)),
            Expression::Value(ValueNode::Integer(42)),
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
            Expression::Value(ValueNode::Integer(42)),
            Expression::Value(ValueNode::Integer(43)),
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
            Expression::Value(ValueNode::Integer(42)),
            Expression::Value(ValueNode::Integer(41)),
        )
        .run(&Context::new())
        .unwrap()
        .as_return_value()
        .unwrap()
        .as_boolean()
        .unwrap());

        assert!(Logic::GreaterOrEqual(
            Expression::Value(ValueNode::Integer(42)),
            Expression::Value(ValueNode::Integer(42)),
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
            Expression::Value(ValueNode::Integer(42)),
            Expression::Value(ValueNode::Integer(43)),
        )
        .run(&Context::new())
        .unwrap()
        .as_return_value()
        .unwrap()
        .as_boolean()
        .unwrap());

        assert!(Logic::LessOrEqual(
            Expression::Value(ValueNode::Integer(42)),
            Expression::Value(ValueNode::Integer(42)),
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
            Expression::Value(ValueNode::Boolean(true)),
            Expression::Value(ValueNode::Boolean(true)),
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
            Expression::Value(ValueNode::Boolean(true)),
            Expression::Value(ValueNode::Boolean(false)),
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
        assert!(Logic::Not(Expression::Value(ValueNode::Boolean(false)))
            .run(&Context::new())
            .unwrap()
            .as_return_value()
            .unwrap()
            .as_boolean()
            .unwrap())
    }
}
