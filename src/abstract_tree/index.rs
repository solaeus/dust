use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    Value,
};

use super::{AbstractTree, Expression, Type, ValueNode};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Index<'src> {
    left: Expression<'src>,
    right: Expression<'src>,
}

impl<'src> Index<'src> {
    pub fn new(left: Expression<'src>, right: Expression<'src>) -> Self {
        Self { left, right }
    }
}

impl<'src> AbstractTree for Index<'src> {
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

    fn run(self, _context: &Context) -> Result<Value, RuntimeError> {
        let left_value = self.left.run(_context)?;
        let right_value = self.right.run(_context)?;

        if let (Some(list), Some(index)) = (left_value.as_list(), right_value.as_integer()) {
            Ok(list
                .get(index as usize)
                .cloned()
                .unwrap_or_else(Value::none))
        } else {
            Err(RuntimeError::ValidationFailure(
                ValidationError::CannotIndexWith(left_value.r#type(), right_value.r#type()),
            ))
        }
    }
}
