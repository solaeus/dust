use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
};

use super::{AbstractTree, Action, Expression, Type, ValueNode, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct MapIndex {
    left: WithPosition<Expression>,
    right: WithPosition<Expression>,
}

impl MapIndex {
    pub fn new(left: WithPosition<Expression>, right: WithPosition<Expression>) -> Self {
        Self { left, right }
    }
}

impl AbstractTree for MapIndex {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        let left_type = self.left.node.expected_type(_context)?;

        if let (
            Expression::Identifier(collection_identifier),
            Expression::Identifier(index_identifier),
        ) = (&self.left.node, &self.right.node)
        {
            let collection = if let Some(collection) = _context.get_value(collection_identifier)? {
                collection
            } else {
                return Err(ValidationError::VariableNotFound(
                    collection_identifier.clone(),
                ));
            };

            if let ValueInner::Map(map) = collection.inner().as_ref() {
                return if let Some(value) = map.get(index_identifier) {
                    Ok(value.r#type())
                } else {
                    Err(ValidationError::PropertyNotFound {
                        identifier: index_identifier.clone(),
                        position: self.right.position,
                    })
                };
            };
        }

        if let (Expression::Value(ValueNode::Map(properties)), Expression::Identifier(identifier)) =
            (&self.left.node, &self.right.node)
        {
            return if let Some(type_result) =
                properties
                    .iter()
                    .find_map(|(property, type_option, expression)| {
                        if property == identifier {
                            if let Some(r#type) = type_option {
                                Some(r#type.node.expected_type(_context))
                            } else {
                                Some(expression.node.expected_type(_context))
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

        Err(ValidationError::CannotIndexWith {
            collection_type: left_type,
            collection_position: self.left.position,
            index_type: self.right.node.expected_type(_context)?,
            index_position: self.right.position,
        })
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        let left_type = self.left.node.expected_type(context)?;

        if let (Expression::Value(ValueNode::Map(_)), Expression::Identifier(_)) =
            (&self.left.node, &self.right.node)
        {
            Ok(())
        } else if let (Expression::Identifier(_), Expression::Identifier(_)) =
            (&self.left.node, &self.right.node)
        {
            Ok(())
        } else {
            Err(ValidationError::CannotIndexWith {
                collection_type: left_type,
                collection_position: self.left.position,
                index_type: self.right.node.expected_type(context)?,
                index_position: self.right.position,
            })
        }
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let action = self.left.node.run(_context)?;
        let collection = if let Action::Return(value) = action {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::InterpreterExpectedReturn(self.left.position),
            ));
        };

        if let (ValueInner::Map(map), Expression::Identifier(identifier)) =
            (collection.inner().as_ref(), &self.right.node)
        {
            let action = map
                .get(identifier)
                .map(|value| Action::Return(value.clone()))
                .unwrap_or(Action::None);

            Ok(action)
        } else {
            Err(RuntimeError::ValidationFailure(
                ValidationError::CannotIndexWith {
                    collection_type: collection.r#type(),
                    collection_position: self.left.position,
                    index_type: self.right.node.expected_type(_context)?,
                    index_position: self.right.position,
                },
            ))
        }
    }
}
