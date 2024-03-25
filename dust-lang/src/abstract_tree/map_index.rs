use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
};

use super::{AbstractNode, Action, Expression, Type, ValueNode, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct MapIndex {
    left: Expression,
    right: Expression,
}

impl MapIndex {
    pub fn new(left: Expression, right: Expression) -> Self {
        Self { left, right }
    }
}

impl AbstractNode for MapIndex {
    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError> {
        if let (
            Expression::Identifier(collection_identifier),
            Expression::Identifier(index_identifier),
        ) = (&self.left, &self.right)
        {
            let collection =
                if let Some(collection) = context.get_value(&collection_identifier.node)? {
                    collection
                } else {
                    return Err(ValidationError::VariableNotFound {
                        identifier: collection_identifier.node.clone(),
                        position: collection_identifier.position,
                    });
                };

            if let ValueInner::Map(map) = collection.inner().as_ref() {
                return if let Some(value) = map.get(&index_identifier.node) {
                    Ok(value.r#type(context)?)
                } else {
                    Err(ValidationError::PropertyNotFound {
                        identifier: index_identifier.node.clone(),
                        position: self.right.position(),
                    })
                };
            };
        }

        if let (
            Expression::Value(WithPosition {
                node: ValueNode::Map(properties),
                ..
            }),
            Expression::Identifier(identifier),
        ) = (&self.left, &self.right)
        {
            return if let Some(type_result) =
                properties
                    .iter()
                    .find_map(|(property, type_option, expression)| {
                        if property == &identifier.node {
                            if let Some(r#type) = type_option {
                                Some(r#type.node.expected_type(context))
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
                node: ValueNode::Structure { fields, .. },
                ..
            }),
            Expression::Identifier(identifier),
        ) = (&self.left, &self.right)
        {
            return if let Some(type_result) = fields.iter().find_map(|(property, expression)| {
                if property == &identifier.node {
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

        Err(ValidationError::CannotIndexWith {
            collection_type: self.left.expected_type(context)?,
            collection_position: self.left.position(),
            index_type: self.right.expected_type(context)?,
            index_position: self.right.position(),
        })
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        let left_type = self.left.expected_type(context)?;

        if let (
            Expression::Value(WithPosition {
                node: ValueNode::Map(_),
                ..
            }),
            Expression::Identifier(_),
        ) = (&self.left, &self.right)
        {
            Ok(())
        } else if let (Expression::Identifier(_), Expression::Identifier(_)) =
            (&self.left, &self.right)
        {
            Ok(())
        } else {
            Err(ValidationError::CannotIndexWith {
                collection_type: left_type,
                collection_position: self.left.position(),
                index_type: self.right.expected_type(context)?,
                index_position: self.right.position(),
            })
        }
    }

    fn run(self, context: &Context) -> Result<Action, RuntimeError> {
        let collection_position = self.left.position();
        let action = self.left.run(context)?;
        let collection = if let Action::Return(value) = action {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::InterpreterExpectedReturn(collection_position),
            ));
        };

        if let (ValueInner::Map(map), Expression::Identifier(identifier)) =
            (collection.inner().as_ref(), &self.right)
        {
            let action = map
                .get(&identifier.node)
                .map(|value| Action::Return(value.clone()))
                .unwrap_or(Action::None);

            Ok(action)
        } else {
            Err(RuntimeError::ValidationFailure(
                ValidationError::CannotIndexWith {
                    collection_type: collection.r#type(context)?,
                    collection_position,
                    index_type: self.right.expected_type(context)?,
                    index_position: self.right.position(),
                },
            ))
        }
    }
}
