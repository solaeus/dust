use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
};

use super::{
    AbstractNode, Action, FunctionCall, ListIndex, Logic, MapIndex, Math, SourcePosition, Type,
    ValueNode, WithPosition,
};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Expression {
    FunctionCall(WithPosition<FunctionCall>),
    Identifier(WithPosition<Identifier>),
    MapIndex(WithPosition<Box<MapIndex>>),
    ListIndex(WithPosition<Box<ListIndex>>),
    Logic(WithPosition<Box<Logic>>),
    Math(WithPosition<Box<Math>>),
    Value(WithPosition<ValueNode>),
}

impl Expression {
    pub fn position(&self) -> SourcePosition {
        match self {
            Expression::FunctionCall(inner) => inner.position,
            Expression::Identifier(inner) => inner.position,
            Expression::MapIndex(inner) => inner.position,
            Expression::ListIndex(inner) => inner.position,
            Expression::Logic(inner) => inner.position,
            Expression::Math(inner) => inner.position,
            Expression::Value(inner) => inner.position,
        }
    }
}

impl AbstractNode for Expression {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match self {
            Expression::FunctionCall(function_call) => function_call.node.expected_type(_context),
            Expression::Identifier(identifier) => {
                if let Some(r#type) = _context.get_type(&identifier.node)? {
                    Ok(r#type)
                } else {
                    Err(ValidationError::VariableNotFound {
                        identifier: identifier.node.clone(),
                        position: identifier.position,
                    })
                }
            }
            Expression::MapIndex(map_index) => map_index.node.expected_type(_context),
            Expression::ListIndex(list_index) => list_index.node.expected_type(_context),
            Expression::Logic(logic) => logic.node.expected_type(_context),
            Expression::Math(math) => math.node.expected_type(_context),
            Expression::Value(value_node) => value_node.node.expected_type(_context),
        }
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        match self {
            Expression::FunctionCall(function_call) => function_call.node.validate(context),
            Expression::Identifier(identifier) => {
                if context.contains(&identifier.node)? {
                    Ok(())
                } else {
                    Err(ValidationError::VariableNotFound {
                        identifier: identifier.node.clone(),
                        position: identifier.position,
                    })
                }
            }
            Expression::MapIndex(map_index) => map_index.node.validate(context),
            Expression::ListIndex(list_index) => list_index.node.validate(context),
            Expression::Logic(logic) => logic.node.validate(context),
            Expression::Math(math) => math.node.validate(context),
            Expression::Value(value_node) => value_node.node.validate(context),
        }
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        match self {
            Expression::FunctionCall(function_call) => function_call.node.run(_context),
            Expression::Identifier(identifier) => {
                if let Some(value) = _context.get_value(&identifier.node)? {
                    Ok(Action::Return(value))
                } else {
                    Err(RuntimeError::ValidationFailure(
                        ValidationError::VariableNotFound {
                            identifier: identifier.node.clone(),
                            position: identifier.position,
                        },
                    ))
                }
            }
            Expression::MapIndex(map_index) => map_index.node.run(_context),
            Expression::ListIndex(list_index) => list_index.node.run(_context),
            Expression::Logic(logic) => logic.node.run(_context),
            Expression::Math(math) => math.node.run(_context),
            Expression::Value(value_node) => value_node.node.run(_context),
        }
    }
}
