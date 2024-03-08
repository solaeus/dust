use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractTree, Action, Expression, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Math {
    Add(Expression, Expression),
    Subtract(Expression, Expression),
    Multiply(Expression, Expression),
    Divide(Expression, Expression),
    Modulo(Expression, Expression),
}

impl AbstractTree for Math {
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

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let value = match self {
            Math::Add(left, right) => {
                let left_value = left.run(_context)?.as_return_value()?;
                let right_value = right.run(_context)?.as_return_value()?;

                left_value.add(&right_value)?
            }
            Math::Subtract(left, right) => {
                let left_value = left.run(_context)?.as_return_value()?;
                let right_value = right.run(_context)?.as_return_value()?;

                left_value.subtract(&right_value)?
            }
            Math::Multiply(left, right) => {
                let left_value = left.run(_context)?.as_return_value()?;
                let right_value = right.run(_context)?.as_return_value()?;

                if let (ValueInner::Integer(left), ValueInner::Integer(right)) =
                    (left_value.inner().as_ref(), right_value.inner().as_ref())
                {
                    Value::integer(left * right)
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedIntegerOrFloat,
                    ));
                }
            }
            Math::Divide(left, right) => {
                let left_value = left.run(_context)?.as_return_value()?;
                let right_value = right.run(_context)?.as_return_value()?;

                if let (ValueInner::Integer(left), ValueInner::Integer(right)) =
                    (left_value.inner().as_ref(), right_value.inner().as_ref())
                {
                    Value::integer(left / right)
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedIntegerOrFloat,
                    ));
                }
            }
            Math::Modulo(left, right) => {
                let left_value = left.run(_context)?.as_return_value()?;
                let right_value = right.run(_context)?.as_return_value()?;

                if let (ValueInner::Integer(left), ValueInner::Integer(right)) =
                    (left_value.inner().as_ref(), right_value.inner().as_ref())
                {
                    Value::integer(left % right)
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedIntegerOrFloat,
                    ));
                }
            }
        };

        Ok(Action::Return(value))
    }
}
