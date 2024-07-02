use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractNode, Evaluation, Expression, SourcePosition, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Math {
    Add(Expression, Expression),
    Subtract(Expression, Expression),
    Multiply(Expression, Expression),
    Divide(Expression, Expression),
    Modulo(Expression, Expression),
}

impl AbstractNode for Math {
    fn define_and_validate(
        &self,
        context: &Context,
        _manage_memory: bool,
    ) -> Result<(), ValidationError> {
        match self {
            Math::Add(left, right) => {
                let left_type = if let Some(r#type) = left.expected_type(context)? {
                    r#type
                } else {
                    return Err(ValidationError::ExpectedValueStatement(left.position()));
                };
                let right_type = if let Some(r#type) = right.expected_type(context)? {
                    r#type
                } else {
                    return Err(ValidationError::ExpectedValueStatement(right.position()));
                };

                if let Type::Integer | Type::Float | Type::String = left_type {
                    if let Type::Integer | Type::Float | Type::String = right_type {
                        Ok(())
                    } else {
                        Err(ValidationError::ExpectedIntegerFloatOrString {
                            actual: right_type,
                            position: right.position(),
                        })
                    }
                } else {
                    Err(ValidationError::ExpectedIntegerFloatOrString {
                        actual: left_type,
                        position: left.position(),
                    })
                }
            }
            Math::Subtract(left, right)
            | Math::Multiply(left, right)
            | Math::Divide(left, right)
            | Math::Modulo(left, right) => {
                let left_type = if let Some(r#type) = left.expected_type(context)? {
                    r#type
                } else {
                    return Err(ValidationError::ExpectedValueStatement(left.position()));
                };
                let right_type = if let Some(r#type) = right.expected_type(context)? {
                    r#type
                } else {
                    return Err(ValidationError::ExpectedValueStatement(right.position()));
                };

                if let Type::Integer | Type::Float = left_type {
                    if let Type::Integer | Type::Float = right_type {
                        Ok(())
                    } else {
                        Err(ValidationError::ExpectedIntegerOrFloat(right.position()))
                    }
                } else {
                    Err(ValidationError::ExpectedIntegerOrFloat(left.position()))
                }
            }
        }
    }

    fn evaluate(
        self,
        _context: &Context,
        _clear_variables: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let run_and_expect_value =
            |position: SourcePosition, expression: Expression| -> Result<Value, RuntimeError> {
                let evaluation = expression.evaluate(&mut _context.clone(), _clear_variables)?;
                let value = if let Some(Evaluation::Return(value)) = evaluation {
                    value
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedValueStatement(position),
                    ));
                };

                Ok(value)
            };

        let value = match self {
            Math::Add(left, right) => {
                let left_position = left.position();
                let left_value = run_and_expect_value(left_position, left)?;
                let right_position = right.position();
                let right_value = run_and_expect_value(right_position, right)?;

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
                    (ValueInner::String(left), ValueInner::String(right)) => {
                        let mut concatenated = String::with_capacity(left.len() + right.len());

                        concatenated.extend(left.chars().chain(right.chars()));

                        Value::string(concatenated)
                    }
                    (ValueInner::Integer(_) | ValueInner::Float(_), _) => {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedIntegerOrFloat(right_position),
                        ))
                    }
                    _ => {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedIntegerOrFloat(left_position),
                        ))
                    }
                }
            }
            Math::Subtract(left, right) => {
                let left_position = left.position();
                let left_value = run_and_expect_value(left_position, left)?;
                let right_position = right.position();
                let right_value = run_and_expect_value(right_position, right)?;

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
                            ValidationError::ExpectedIntegerOrFloat(right_position),
                        ))
                    }
                    _ => {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedIntegerOrFloat(left_position),
                        ))
                    }
                }
            }
            Math::Multiply(left, right) => {
                let left_position = left.position();
                let left_value = run_and_expect_value(left_position, left)?;
                let right_position = right.position();
                let right_value = run_and_expect_value(right_position, right)?;

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
                            ValidationError::ExpectedIntegerOrFloat(right_position).into(),
                        ))
                    }
                    _ => {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedIntegerOrFloat(left_position),
                        ))
                    }
                }
            }
            Math::Divide(left, right) => {
                let left_position = left.position();
                let left_value = run_and_expect_value(left_position, left)?;
                let right_position = right.position();
                let right_value = run_and_expect_value(right_position, right)?;

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
                            ValidationError::ExpectedIntegerOrFloat(right_position).into(),
                        ))
                    }
                    _ => {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedIntegerOrFloat(left_position),
                        ))
                    }
                }
            }
            Math::Modulo(left, right) => {
                let left_position = left.position();
                let left_value = run_and_expect_value(left_position, left)?;
                let right_position = right.position();
                let right_value = run_and_expect_value(right_position, right)?;

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
                            ValidationError::ExpectedIntegerOrFloat(right_position).into(),
                        ))
                    }
                    _ => {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedIntegerOrFloat(left_position),
                        ))
                    }
                }
            }
        };

        Ok(Some(Evaluation::Return(value)))
    }

    fn expected_type(&self, _context: &Context) -> Result<Option<Type>, ValidationError> {
        match self {
            Math::Add(left, right)
            | Math::Subtract(left, right)
            | Math::Multiply(left, right)
            | Math::Divide(left, right)
            | Math::Modulo(left, right) => {
                let left_type = if let Some(r#type) = left.expected_type(_context)? {
                    r#type
                } else {
                    return Err(ValidationError::ExpectedValueStatement(left.position()));
                };
                let right_type = if let Some(r#type) = right.expected_type(_context)? {
                    r#type
                } else {
                    return Err(ValidationError::ExpectedValueStatement(right.position()));
                };

                if let Type::Float = left_type {
                    return Ok(Some(Type::Float));
                }

                if let Type::Float = right_type {
                    return Ok(Some(Type::Float));
                }

                Ok(Some(left_type))
            }
        }
    }
}

impl Display for Math {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Math::Add(left, right) => write!(f, "{left} + {right}"),
            Math::Subtract(left, right) => write!(f, "{left} - {right}"),
            Math::Multiply(left, right) => write!(f, "{left} * {right}"),
            Math::Divide(left, right) => write!(f, "{left} / {right}"),
            Math::Modulo(left, right) => write!(f, "{left} % {right}"),
        }
    }
}
