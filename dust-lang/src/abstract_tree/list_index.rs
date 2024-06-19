use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{Evaluate, Evaluation, ExpectedType, Expression, Type, ValueNode, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ListIndex {
    collection: Expression,
    index: Expression,
}

impl ListIndex {
    pub fn new(left: Expression, right: Expression) -> Self {
        Self {
            collection: left,
            index: right,
        }
    }
}

impl Evaluate for ListIndex {
    fn validate(&self, context: &mut Context, _manage_memory: bool) -> Result<(), ValidationError> {
        self.collection.validate(context, _manage_memory)?;
        self.index.validate(context, _manage_memory)?;

        let collection_type = self.collection.expected_type(context)?;
        let index_type = self.index.expected_type(context)?;

        match collection_type {
            Type::List {
                length: _,
                item_type: _,
            } => {
                if index_type == Type::Integer {
                    Ok(())
                } else {
                    Err(ValidationError::CannotIndexWith {
                        collection_type,
                        collection_position: self.collection.position(),
                        index_type,
                        index_position: self.index.position(),
                    })
                }
            }
            Type::ListOf(_) => todo!(),
            _ => Err(ValidationError::CannotIndex {
                r#type: collection_type,
                position: self.collection.position(),
            }),
        }
    }

    fn evaluate(
        self,
        context: &mut Context,
        _clear_variables: bool,
    ) -> Result<Evaluation, RuntimeError> {
        let left_position = self.collection.position();
        let left_action = self.collection.evaluate(context, _clear_variables)?;
        let left_value = if let Evaluation::Return(value) = left_action {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::InterpreterExpectedReturn(left_position),
            ));
        };
        let right_position = self.index.position();
        let right_action = self.index.evaluate(context, _clear_variables)?;
        let right_value = if let Evaluation::Return(value) = right_action {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::InterpreterExpectedReturn(right_position),
            ));
        };

        if let (Some(list), Some(index)) = (left_value.as_list(), right_value.as_integer()) {
            let found_item = list.get(index as usize);

            if let Some(item) = found_item {
                Ok(Evaluation::Return(item.clone()))
            } else {
                Ok(Evaluation::None)
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

impl ExpectedType for ListIndex {
    fn expected_type(&self, _context: &mut Context) -> Result<Type, ValidationError> {
        let left_type = self.collection.expected_type(_context)?;

        if let (
            Expression::Value(WithPosition {
                node: ValueNode::List(expression_list),
                ..
            }),
            Expression::Value(WithPosition {
                node: ValueNode::Integer(index),
                ..
            }),
        ) = (&self.collection, &self.index)
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
                position: self.collection.position(),
            })
        }
    }
}
