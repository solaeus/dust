use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Expression, Type, ValueNode, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Index {
    left: WithPosition<Expression>,
    right: WithPosition<Expression>,
}

impl Index {
    pub fn new(left: WithPosition<Expression>, right: WithPosition<Expression>) -> Self {
        Self { left, right }
    }
}

impl AbstractTree for Index {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        let left_type = self.left.node.expected_type(_context)?;

        if let (
            Expression::Value(ValueNode::List(expression_list)),
            Expression::Value(ValueNode::Integer(index)),
        ) = (&self.left.node, &self.right.node)
        {
            let expression = if let Some(expression) = expression_list.get(*index as usize) {
                expression
            } else {
                return Ok(Type::None);
            };

            expression.node.expected_type(_context)
        } else {
            Err(ValidationError::CannotIndex {
                r#type: left_type,
                position: self.left.position,
            })
        }
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        let left_type = self.left.node.expected_type(context)?;

        match left_type {
            Type::List => todo!(),
            Type::ListOf(_) => todo!(),
            Type::ListExact(_) => {
                let right_type = self.right.node.expected_type(context)?;

                if let Type::Integer = right_type {
                    Ok(())
                } else {
                    Err(ValidationError::CannotIndexWith(left_type, right_type))
                }
            }
            _ => Err(ValidationError::CannotIndex {
                r#type: left_type,
                position: self.left.position,
            }),
        }
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let left_value = self.left.node.run(_context)?.as_return_value()?;
        let right_value = self.right.node.run(_context)?.as_return_value()?;

        if let (Some(list), Some(index)) = (left_value.as_list(), right_value.as_integer()) {
            let found_item = list.get(index as usize);

            if let Some(item) = found_item {
                Ok(Action::Return(item.clone()))
            } else {
                Ok(Action::None)
            }
        } else {
            Err(RuntimeError::ValidationFailure(
                ValidationError::CannotIndexWith(left_value.r#type(), right_value.r#type()),
            ))
        }
    }
}
