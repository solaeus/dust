use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractNode, Action, Expression, Type, ValueNode, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct ListIndex {
    left: Expression,
    right: Expression,
}

impl ListIndex {
    pub fn new(left: Expression, right: Expression) -> Self {
        Self { left, right }
    }
}

impl AbstractNode for ListIndex {
    fn expected_type(&self, _context: &mut Context) -> Result<Type, ValidationError> {
        let left_type = self.left.expected_type(_context)?;

        if let (
            Expression::Value(WithPosition {
                item: ValueNode::List(expression_list),
                ..
            }),
            Expression::Value(WithPosition {
                item: ValueNode::Integer(index),
                ..
            }),
        ) = (&self.left, &self.right)
        {
            let expression = if let Some(expression) = expression_list.get(*index as usize) {
                expression
            } else {
                return Ok(Type::None);
            };

            expression.expected_type(_context)
        } else {
            Err(ValidationError::CannotIndex {
                r#type: left_type,
                position: self.left.position(),
            })
        }
    }

    fn validate(&self, context: &mut Context, _manage_memory: bool) -> Result<(), ValidationError> {
        self.left.validate(context, _manage_memory)?;
        self.right.validate(context, _manage_memory)?;

        let left_type = self.left.expected_type(context)?;

        match left_type {
            Type::List => todo!(),
            Type::ListOf(_) => todo!(),
            Type::ListExact(_) => {
                let right_type = self.right.expected_type(context)?;

                if let Type::Integer = right_type {
                    Ok(())
                } else {
                    Err(ValidationError::CannotIndexWith {
                        collection_type: left_type,
                        collection_position: self.left.position(),
                        index_type: right_type,
                        index_position: self.right.position(),
                    })
                }
            }
            _ => Err(ValidationError::CannotIndex {
                r#type: left_type,
                position: self.left.position(),
            }),
        }
    }

    fn run(self, context: &mut Context, _clear_variables: bool) -> Result<Action, RuntimeError> {
        let left_position = self.left.position();
        let left_action = self.left.run(context, _clear_variables)?;
        let left_value = if let Action::Return(value) = left_action {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::InterpreterExpectedReturn(left_position),
            ));
        };
        let right_position = self.right.position();
        let right_action = self.right.run(context, _clear_variables)?;
        let right_value = if let Action::Return(value) = right_action {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::InterpreterExpectedReturn(right_position),
            ));
        };

        if let (Some(list), Some(index)) = (left_value.as_list(), right_value.as_integer()) {
            let found_item = list.get(index as usize);

            if let Some(item) = found_item {
                Ok(Action::Return(item.item.clone()))
            } else {
                Ok(Action::None)
            }
        } else {
            Err(RuntimeError::ValidationFailure(
                ValidationError::CannotIndexWith {
                    collection_type: left_value.r#type(context)?,
                    collection_position: left_position,
                    index_type: right_value.r#type(context)?,
                    index_position: right_position,
                },
            ))
        }
    }
}
