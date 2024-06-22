use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractNode, Evaluation, Expression, Type, ValueNode, WithPosition};

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

impl AbstractNode for ListIndex {
    fn define_types(&self, context: &Context) -> Result<(), ValidationError> {
        self.collection.define_types(context)?;
        self.index.define_types(context)
    }

    fn validate(&self, context: &Context, _manage_memory: bool) -> Result<(), ValidationError> {
        self.collection.validate(context, _manage_memory)?;
        self.index.validate(context, _manage_memory)?;

        let collection_type = if let Some(r#type) = self.index.expected_type(context)? {
            r#type
        } else {
            return Err(ValidationError::ExpectedExpression(
                self.collection.position(),
            ));
        };
        let index_type = if let Some(r#type) = self.index.expected_type(context)? {
            r#type
        } else {
            return Err(ValidationError::ExpectedExpression(self.index.position()));
        };

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
        context: &Context,
        _clear_variables: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let left_position = self.collection.position();
        let left_evaluation = self.collection.evaluate(context, _clear_variables)?;
        let left_value = if let Some(Evaluation::Return(value)) = left_evaluation {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::ExpectedExpression(left_position),
            ));
        };
        let right_position = self.index.position();
        let right_evaluation = self.index.evaluate(context, _clear_variables)?;
        let right_value = if let Some(Evaluation::Return(value)) = right_evaluation {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::ExpectedExpression(right_position),
            ));
        };

        if let (Some(list), Some(index)) = (left_value.as_list(), right_value.as_integer()) {
            let found_item = list.get(index as usize);

            if let Some(item) = found_item {
                Ok(Some(Evaluation::Return(item.clone())))
            } else {
                Ok(None)
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

    fn expected_type(&self, _context: &Context) -> Result<Option<Type>, ValidationError> {
        let left_type = if let Some(r#type) = self.collection.expected_type(_context)? {
            r#type
        } else {
            return Err(ValidationError::ExpectedExpression(
                self.collection.position(),
            ));
        };

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
                return Ok(None);
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
