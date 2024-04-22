use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
};

use super::{
    AbstractNode, Action, BuiltInFunctionCall, FunctionCall, ListIndex, Logic, MapIndex, Math,
    SourcePosition, Type, ValueNode, WithPosition,
};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Expression {
    BuiltInFunctionCall(WithPosition<Box<BuiltInFunctionCall>>),
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
            Expression::BuiltInFunctionCall(inner) => inner.position,
        }
    }
}

impl AbstractNode for Expression {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match self {
            Expression::FunctionCall(function_call) => function_call.item.expected_type(_context),
            Expression::Identifier(identifier) => {
                if let Some(r#type) = _context.get_type(&identifier.item)? {
                    Ok(r#type)
                } else {
                    Err(ValidationError::VariableNotFound {
                        identifier: identifier.item.clone(),
                        position: identifier.position,
                    })
                }
            }
            Expression::MapIndex(map_index) => map_index.item.expected_type(_context),
            Expression::ListIndex(list_index) => list_index.item.expected_type(_context),
            Expression::Logic(logic) => logic.item.expected_type(_context),
            Expression::Math(math) => math.item.expected_type(_context),
            Expression::Value(value_node) => value_node.item.expected_type(_context),
            Expression::BuiltInFunctionCall(built_in_function_call) => {
                built_in_function_call.item.expected_type(_context)
            }
        }
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        match self {
            Expression::FunctionCall(function_call) => function_call.item.validate(context),
            Expression::Identifier(identifier) => {
                let found = context.add_expected_use(&identifier.item)?;

                if found {
                    Ok(())
                } else {
                    Err(ValidationError::VariableNotFound {
                        identifier: identifier.item.clone(),
                        position: identifier.position,
                    })
                }
            }
            Expression::MapIndex(map_index) => map_index.item.validate(context),
            Expression::ListIndex(list_index) => list_index.item.validate(context),
            Expression::Logic(logic) => logic.item.validate(context),
            Expression::Math(math) => math.item.validate(context),
            Expression::Value(value_node) => value_node.item.validate(context),
            Expression::BuiltInFunctionCall(built_in_function_call) => {
                built_in_function_call.item.validate(context)
            }
        }
    }

    fn run(self, context: &mut Context, _clear_variables: bool) -> Result<Action, RuntimeError> {
        match self {
            Expression::FunctionCall(function_call) => {
                function_call.item.run(context, _clear_variables)
            }
            Expression::Identifier(identifier) => {
                if let Some(value) = context.use_value(&identifier.item)? {
                    Ok(Action::Return(value))
                } else {
                    Err(RuntimeError::ValidationFailure(
                        ValidationError::VariableNotFound {
                            identifier: identifier.item.clone(),
                            position: identifier.position,
                        },
                    ))
                }
            }
            Expression::MapIndex(map_index) => map_index.item.run(context, _clear_variables),
            Expression::ListIndex(list_index) => list_index.item.run(context, _clear_variables),
            Expression::Logic(logic) => logic.item.run(context, _clear_variables),
            Expression::Math(math) => math.item.run(context, _clear_variables),
            Expression::Value(value_node) => value_node.item.run(context, _clear_variables),
            Expression::BuiltInFunctionCall(built_in_function_call) => {
                built_in_function_call.item.run(context, _clear_variables)
            }
        }
    }
}
