use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractTree, Expression, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Math<'src> {
    Add(Expression<'src>, Expression<'src>),
    Subtract(Expression<'src>, Expression<'src>),
    Multiply(Expression<'src>, Expression<'src>),
    Divide(Expression<'src>, Expression<'src>),
    Modulo(Expression<'src>, Expression<'src>),
}

impl<'src> AbstractTree for Math<'src> {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match self {
            Math::Add(left, _)
            | Math::Subtract(left, _)
            | Math::Multiply(left, _)
            | Math::Divide(left, _)
            | Math::Modulo(left, _) => left.expected_type(_context),
        }
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        match self {
            Math::Add(left, right)
            | Math::Subtract(left, right)
            | Math::Multiply(left, right)
            | Math::Divide(left, right)
            | Math::Modulo(left, right) => {
                let left_type = left.expected_type(context)?;
                let right_type = right.expected_type(context)?;

                match (left_type, right_type) {
                    (Type::Integer, Type::Integer)
                    | (Type::Float, Type::Float)
                    | (Type::Integer, Type::Float)
                    | (Type::Float, Type::Integer) => Ok(()),
                    _ => Err(ValidationError::ExpectedIntegerOrFloat),
                }
            }
        }
    }

    fn run(self, context: &Context) -> Result<Value, RuntimeError> {
        match self {
            Math::Add(left, right) => {
                let left_value = left.run(context)?;
                let right_value = right.run(context)?;

                if let (ValueInner::Integer(left), ValueInner::Integer(right)) =
                    (left_value.inner().as_ref(), right_value.inner().as_ref())
                {
                    Ok(Value::integer(left + right))
                } else {
                    Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedIntegerOrFloat,
                    ))
                }
            }
            Math::Subtract(left, right) => {
                let left_value = left.run(context)?;
                let right_value = right.run(context)?;

                if let (ValueInner::Integer(left), ValueInner::Integer(right)) =
                    (left_value.inner().as_ref(), right_value.inner().as_ref())
                {
                    Ok(Value::integer(left - right))
                } else {
                    Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedIntegerOrFloat,
                    ))
                }
            }
            Math::Multiply(left, right) => {
                let left_value = left.run(context)?;
                let right_value = right.run(context)?;

                if let (ValueInner::Integer(left), ValueInner::Integer(right)) =
                    (left_value.inner().as_ref(), right_value.inner().as_ref())
                {
                    Ok(Value::integer(left * right))
                } else {
                    Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedIntegerOrFloat,
                    ))
                }
            }
            Math::Divide(left, right) => {
                let left_value = left.run(context)?;
                let right_value = right.run(context)?;

                if let (ValueInner::Integer(left), ValueInner::Integer(right)) =
                    (left_value.inner().as_ref(), right_value.inner().as_ref())
                {
                    Ok(Value::integer(left / right))
                } else {
                    Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedIntegerOrFloat,
                    ))
                }
            }
            Math::Modulo(left, right) => {
                let left_value = left.run(context)?;
                let right_value = right.run(context)?;

                if let (ValueInner::Integer(left), ValueInner::Integer(right)) =
                    (left_value.inner().as_ref(), right_value.inner().as_ref())
                {
                    Ok(Value::integer(left % right))
                } else {
                    Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedIntegerOrFloat,
                    ))
                }
            }
        }
    }
}
