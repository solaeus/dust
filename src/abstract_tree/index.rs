use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    Value,
};

use super::{AbstractTree, Action, Expression, Type, ValueNode};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Index {
    left: Expression,
    right: Expression,
}

impl Index {
    pub fn new(left: Expression, right: Expression) -> Self {
        Self { left, right }
    }
}

impl AbstractTree for Index {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        let left_type = self.left.expected_type(_context)?;

        if let (
            Expression::Value(ValueNode::List(expression_list)),
            Expression::Value(ValueNode::Integer(index)),
        ) = (&self.left, &self.right)
        {
            let expression = if let Some(expression) = expression_list.get(*index as usize) {
                expression
            } else {
                return Ok(Type::None);
            };

            expression.expected_type(_context)
        } else {
            Err(ValidationError::CannotIndex(left_type))
        }
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        let left_type = self.left.expected_type(context)?;

        match left_type {
            Type::List => todo!(),
            Type::ListOf(_) => todo!(),
            Type::ListExact(_) => {
                let right_type = self.right.expected_type(context)?;

                if let Type::Integer = right_type {
                    Ok(())
                } else {
                    Err(ValidationError::CannotIndexWith(left_type, right_type))
                }
            }
            _ => Err(ValidationError::CannotIndex(left_type)),
        }
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let left_value = self.left.run(_context)?.as_return_value()?;
        let right_value = self.right.run(_context)?.as_return_value()?;

        if let (Some(list), Some(index)) = (left_value.as_list(), right_value.as_integer()) {
            Ok(Action::Return(
                list.get(index as usize)
                    .cloned()
                    .unwrap_or_else(Value::none),
            ))
        } else {
            Err(RuntimeError::ValidationFailure(
                ValidationError::CannotIndexWith(left_value.r#type(), right_value.r#type()),
            ))
        }
    }
}
