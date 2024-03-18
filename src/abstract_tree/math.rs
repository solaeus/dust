use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractTree, Action, Expression, SourcePosition, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Math {
    Add(WithPosition<Expression>, WithPosition<Expression>),
    Subtract(WithPosition<Expression>, WithPosition<Expression>),
    Multiply(WithPosition<Expression>, WithPosition<Expression>),
    Divide(WithPosition<Expression>, WithPosition<Expression>),
    Modulo(WithPosition<Expression>, WithPosition<Expression>),
}

impl AbstractTree for Math {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match self {
            Math::Add(left, _)
            | Math::Subtract(left, _)
            | Math::Multiply(left, _)
            | Math::Divide(left, _)
            | Math::Modulo(left, _) => left.node.expected_type(_context),
        }
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        match self {
            Math::Add(left, right)
            | Math::Subtract(left, right)
            | Math::Multiply(left, right)
            | Math::Divide(left, right)
            | Math::Modulo(left, right) => {
                let left_type = left.node.expected_type(context)?;
                let right_type = right.node.expected_type(context)?;

                if let Type::Integer | Type::Float = left_type {
                    if let Type::Integer | Type::Float = right_type {
                        Ok(())
                    } else {
                        Err(ValidationError::ExpectedIntegerOrFloat(right.position))
                    }
                } else {
                    Err(ValidationError::ExpectedIntegerOrFloat(left.position))
                }
            }
        }
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let run_and_expect_value =
            |expression: Expression, position: SourcePosition| -> Result<Value, RuntimeError> {
                let action = expression.run(_context)?;
                let value = if let Action::Return(value) = action {
                    value
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::InterpreterExpectedReturn(position),
                    ));
                };

                Ok(value)
            };

        let value = match self {
            Math::Add(left, right) => {
                let left_value = run_and_expect_value(left.node, left.position)?;
                let right_value = run_and_expect_value(right.node, right.position)?;

                match (left_value.inner().as_ref(), right_value.inner().as_ref()) {
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
                    (ValueInner::Integer(_) | ValueInner::Float(_), _) => {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedIntegerOrFloat(right.position),
                        ))
                    }
                    _ => {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedIntegerOrFloat(left.position),
                        ))
                    }
                }
            }
            Math::Subtract(left, right) => {
                let left_value = run_and_expect_value(left.node, left.position)?;
                let right_value = run_and_expect_value(right.node, right.position)?;

                match (left_value.inner().as_ref(), right_value.inner().as_ref()) {
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
                    (ValueInner::Integer(_) | ValueInner::Float(_), _) => {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedIntegerOrFloat(right.position),
                        ))
                    }
                    _ => {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedIntegerOrFloat(left.position),
                        ))
                    }
                }
            }
            Math::Multiply(left, right) => {
                let left_value = run_and_expect_value(left.node, left.position)?;
                let right_value = run_and_expect_value(right.node, right.position)?;

                match (left_value.inner().as_ref(), right_value.inner().as_ref()) {
                    (ValueInner::Integer(left), ValueInner::Integer(right)) => {
                        let product = left.saturating_mul(*right);

                        Value::integer(product)
                    }
                    (ValueInner::Float(left), ValueInner::Float(right)) => {
                        let product = left * right;

                        Value::float(product)
                    }
                    (ValueInner::Float(left), ValueInner::Integer(right)) => {
                        let product = left * *right as f64;

                        Value::float(product)
                    }
                    (ValueInner::Integer(left), ValueInner::Float(right)) => {
                        let product = *left as f64 * right;

                        Value::float(product)
                    }
                    (ValueInner::Integer(_) | ValueInner::Float(_), _) => {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedIntegerOrFloat(right.position).into(),
                        ))
                    }
                    _ => {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedIntegerOrFloat(left.position),
                        ))
                    }
                }
            }
            Math::Divide(left, right) => {
                let left_value = run_and_expect_value(left.node, left.position)?;
                let right_value = run_and_expect_value(right.node, right.position)?;

                match (left_value.inner().as_ref(), right_value.inner().as_ref()) {
                    (ValueInner::Integer(left), ValueInner::Integer(right)) => {
                        let quotient = left.saturating_div(*right);

                        Value::integer(quotient)
                    }
                    (ValueInner::Float(left), ValueInner::Float(right)) => {
                        let quotient = left / right;

                        Value::float(quotient)
                    }
                    (ValueInner::Float(left), ValueInner::Integer(right)) => {
                        let quotient = left / *right as f64;

                        Value::float(quotient)
                    }
                    (ValueInner::Integer(left), ValueInner::Float(right)) => {
                        let quotient = *left as f64 / right;

                        Value::float(quotient)
                    }
                    (ValueInner::Integer(_) | ValueInner::Float(_), _) => {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedIntegerOrFloat(right.position).into(),
                        ))
                    }
                    _ => {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedIntegerOrFloat(left.position),
                        ))
                    }
                }
            }
            Math::Modulo(left, right) => {
                let left_value = run_and_expect_value(left.node, left.position)?;
                let right_value = run_and_expect_value(right.node, right.position)?;

                match (left_value.inner().as_ref(), right_value.inner().as_ref()) {
                    (ValueInner::Integer(left), ValueInner::Integer(right)) => {
                        let remainder = left % right;

                        Value::integer(remainder)
                    }
                    (ValueInner::Float(left), ValueInner::Float(right)) => {
                        let remainder = left % right;

                        Value::float(remainder)
                    }
                    (ValueInner::Float(left), ValueInner::Integer(right)) => {
                        let remainder = left % *right as f64;

                        Value::float(remainder)
                    }
                    (ValueInner::Integer(left), ValueInner::Float(right)) => {
                        let remainder = *left as f64 % right;

                        Value::float(remainder)
                    }
                    (ValueInner::Integer(_) | ValueInner::Float(_), _) => {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedIntegerOrFloat(right.position).into(),
                        ))
                    }
                    _ => {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedIntegerOrFloat(left.position),
                        ))
                    }
                }
            }
        };

        Ok(Action::Return(value))
    }
}
