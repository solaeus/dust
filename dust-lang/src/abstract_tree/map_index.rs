use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
};

use super::{Evaluate, Evaluation, ExpectedType, Expression, Type, ValueNode, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MapIndex {
    collection: Expression,
    index: Expression,
}

impl MapIndex {
    pub fn new(left: Expression, right: Expression) -> Self {
        Self {
            collection: left,
            index: right,
        }
    }
}

impl Evaluate for MapIndex {
    fn validate(
        &self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<(), ValidationError> {
        self.collection.validate(_context, _manage_memory)
    }

    fn evaluate(
        self,
        context: &mut Context,
        _manage_memory: bool,
    ) -> Result<Evaluation, RuntimeError> {
        let collection_position = self.collection.position();
        let action = self.collection.evaluate(context, _manage_memory)?;
        let collection = if let Evaluation::Return(value) = action {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::InterpreterExpectedReturn(collection_position),
            ));
        };

        if let (ValueInner::Map(map), Expression::Identifier(index)) =
            (collection.inner().as_ref(), self.index)
        {
            let action = map
                .get(&index.node)
                .map(|value| Evaluation::Return(value.clone()))
                .unwrap_or(Evaluation::None);

            Ok(action)
        } else {
            Err(RuntimeError::ValidationFailure(
                ValidationError::CannotIndex {
                    r#type: collection.r#type(context)?,
                    position: collection_position,
                },
            ))
        }
    }
}

impl ExpectedType for MapIndex {
    fn expected_type(&self, context: &mut Context) -> Result<Type, ValidationError> {
        if let (Expression::Identifier(collection), Expression::Identifier(index)) =
            (&self.collection, &self.index)
        {
            let collection = if let Some(collection) = context.get_value(&collection.node)? {
                collection
            } else {
                return Err(ValidationError::VariableNotFound {
                    identifier: collection.node.clone(),
                    position: collection.position,
                });
            };

            if let ValueInner::Map(map) = collection.inner().as_ref() {
                return if let Some(value) = map.get(&index.node) {
                    Ok(value.r#type(context)?)
                } else {
                    Err(ValidationError::PropertyNotFound {
                        identifier: index.node.clone(),
                        position: index.position,
                    })
                };
            };
        }

        if let (
            Expression::Value(WithPosition {
                node: ValueNode::Map(properties),
                ..
            }),
            Expression::Identifier(index),
        ) = (&self.collection, &self.index)
        {
            for (property, constructor_option, expression) in properties {
                if property == &index.node {
                    return if let Some(constructor) = constructor_option {
                        let r#type = constructor.clone().construct(&context)?;

                        Ok(r#type)
                    } else {
                        Ok(expression.expected_type(context)?)
                    };
                }
            }

            return Ok(Type::None);
        }

        if let (
            Expression::Value(WithPosition {
                node: ValueNode::Structure { fields, .. },
                ..
            }),
            Expression::Identifier(index),
        ) = (&self.collection, &self.index)
        {
            return if let Some(type_result) = fields.iter().find_map(|(property, expression)| {
                if property.node == index.node {
                    Some(expression.expected_type(context))
                } else {
                    None
                }
            }) {
                type_result
            } else {
                Ok(Type::None)
            };
        }

        Err(ValidationError::CannotIndex {
            r#type: self.collection.expected_type(context)?,
            position: self.collection.position(),
        })
    }
}
