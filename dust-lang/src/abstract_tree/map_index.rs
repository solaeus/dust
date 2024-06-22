use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
};

use super::{AbstractNode, Evaluation, Expression, Type, ValueNode, WithPosition};

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

impl AbstractNode for MapIndex {
    fn define_types(&self, _context: &Context) -> Result<(), ValidationError> {
        self.collection.define_types(_context)?;
        self.index.define_types(_context)
    }

    fn validate(&self, _context: &Context, _manage_memory: bool) -> Result<(), ValidationError> {
        self.collection.validate(_context, _manage_memory)
    }

    fn evaluate(
        self,
        context: &Context,
        _manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let collection_position = self.collection.position();
        let action = self.collection.evaluate(context, _manage_memory)?;
        let collection = if let Evaluation::Return(value) = action {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::ExpectedExpression(collection_position),
            ));
        };

        if let (ValueInner::Map(map), Expression::Identifier(index)) =
            (collection.inner().as_ref(), self.index)
        {
            let evaluation = map
                .get(&index.node)
                .map(|value| Some(Evaluation::Return(value.clone())));

            Ok(evaluation)
        } else {
            Err(RuntimeError::ValidationFailure(
                ValidationError::CannotIndex {
                    r#type: collection.r#type(context)?,
                    position: collection_position,
                },
            ))
        }
    }

    fn expected_type(&self, context: &Context) -> Result<Option<Type>, ValidationError> {
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
                    Ok(Some(value.r#type(context)?))
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

                        Ok(Some(r#type))
                    } else {
                        expression.expected_type(context)
                    };
                }
            }

            return Ok(Type::Void);
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
                Ok(Type::Void)
            };
        }

        Err(ValidationError::CannotIndex {
            r#type: self.collection.expected_type(context)?,
            position: self.collection.position(),
        })
    }
}
