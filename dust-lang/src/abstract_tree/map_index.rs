use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
    value::ValueInner,
};

use super::{AbstractNode, Action, Expression, Type, ValueNode, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct MapIndex {
    collection: Expression,
    index: WithPosition<Identifier>,
}

impl MapIndex {
    pub fn new(left: Expression, right: WithPosition<Identifier>) -> Self {
        Self {
            collection: left,
            index: right,
        }
    }
}

impl AbstractNode for MapIndex {
    fn expected_type(&self, context: &mut Context) -> Result<Type, ValidationError> {
        if let (Expression::Identifier(collection_identifier), index) =
            (&self.collection, &self.index)
        {
            let collection =
                if let Some(collection) = context.get_value(&collection_identifier.item)? {
                    collection
                } else {
                    return Err(ValidationError::VariableNotFound {
                        identifier: collection_identifier.item.clone(),
                        position: collection_identifier.position,
                    });
                };

            if let ValueInner::Map(map) = collection.inner().as_ref() {
                return if let Some(value) = map.get(&index.item) {
                    Ok(value.r#type(context)?)
                } else {
                    Err(ValidationError::PropertyNotFound {
                        identifier: index.item.clone(),
                        position: index.position,
                    })
                };
            };
        }

        if let (
            Expression::Value(WithPosition {
                item: ValueNode::Map(properties),
                ..
            }),
            index,
        ) = (&self.collection, &self.index)
        {
            return if let Some(type_result) =
                properties
                    .iter()
                    .find_map(|(property, type_option, expression)| {
                        if property == &index.item {
                            if let Some(r#type) = type_option {
                                Some(r#type.item.expected_type(context))
                            } else {
                                Some(expression.expected_type(context))
                            }
                        } else {
                            None
                        }
                    })
            {
                type_result
            } else {
                Ok(Type::None)
            };
        }

        if let (
            Expression::Value(WithPosition {
                item: ValueNode::Structure { fields, .. },
                ..
            }),
            index,
        ) = (&self.collection, &self.index)
        {
            return if let Some(type_result) = fields.iter().find_map(|(property, expression)| {
                if property == &index.item {
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

    fn validate(
        &self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<(), ValidationError> {
        self.collection.validate(_context, _manage_memory)
    }

    fn run(self, context: &mut Context, _manage_memory: bool) -> Result<Action, RuntimeError> {
        let collection_position = self.collection.position();
        let action = self.collection.run(context, _manage_memory)?;
        let collection = if let Action::Return(value) = action {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::InterpreterExpectedReturn(collection_position),
            ));
        };

        if let ValueInner::Map(map) = collection.inner().as_ref() {
            let action = map
                .get(&self.index.item)
                .map(|value| Action::Return(value.clone()))
                .unwrap_or(Action::None);

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
