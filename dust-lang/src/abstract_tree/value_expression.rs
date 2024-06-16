use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
};

use super::{
    AbstractNode, Action, As, BuiltInFunctionCall, ExpectedType, FunctionCall, ListIndex, Logic,
    MapIndex, Math, SourcePosition, Type, ValueNode, WithPosition,
};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ValueExpression {
    As(WithPosition<Box<As>>),
    BuiltInFunctionCall(WithPosition<Box<BuiltInFunctionCall>>),
    FunctionCall(WithPosition<FunctionCall>),
    Identifier(WithPosition<Identifier>),
    MapIndex(WithPosition<Box<MapIndex>>),
    ListIndex(WithPosition<Box<ListIndex>>),
    Logic(WithPosition<Box<Logic>>),
    Math(WithPosition<Box<Math>>),
    Value(WithPosition<ValueNode>),
}

impl ValueExpression {
    pub fn position(&self) -> SourcePosition {
        match self {
            ValueExpression::As(inner) => inner.position,
            ValueExpression::FunctionCall(inner) => inner.position,
            ValueExpression::Identifier(inner) => inner.position,
            ValueExpression::MapIndex(inner) => inner.position,
            ValueExpression::ListIndex(inner) => inner.position,
            ValueExpression::Logic(inner) => inner.position,
            ValueExpression::Math(inner) => inner.position,
            ValueExpression::Value(inner) => inner.position,
            ValueExpression::BuiltInFunctionCall(inner) => inner.position,
        }
    }
}

impl AbstractNode for ValueExpression {
    fn validate(&self, context: &mut Context, manage_memory: bool) -> Result<(), ValidationError> {
        match self {
            ValueExpression::As(r#as) => r#as.node.validate(context, manage_memory),
            ValueExpression::FunctionCall(function_call) => {
                function_call.node.validate(context, manage_memory)
            }
            ValueExpression::Identifier(identifier) => {
                let found = if manage_memory {
                    context.add_expected_use(&identifier.node)?
                } else {
                    context.contains(&identifier.node)?
                };

                if found {
                    Ok(())
                } else {
                    Err(ValidationError::VariableNotFound {
                        identifier: identifier.node.clone(),
                        position: identifier.position,
                    })
                }
            }
            ValueExpression::MapIndex(map_index) => map_index.node.validate(context, manage_memory),
            ValueExpression::ListIndex(list_index) => {
                list_index.node.validate(context, manage_memory)
            }
            ValueExpression::Logic(logic) => logic.node.validate(context, manage_memory),
            ValueExpression::Math(math) => math.node.validate(context, manage_memory),
            ValueExpression::Value(value_node) => value_node.node.validate(context, manage_memory),
            ValueExpression::BuiltInFunctionCall(built_in_function_call) => {
                built_in_function_call.node.validate(context, manage_memory)
            }
        }
    }

    fn run(self, context: &mut Context, manage_memory: bool) -> Result<Action, RuntimeError> {
        match self {
            ValueExpression::As(r#as) => r#as.node.run(context, manage_memory),
            ValueExpression::FunctionCall(function_call) => {
                function_call.node.run(context, manage_memory)
            }
            ValueExpression::Identifier(identifier) => {
                let value_option = if manage_memory {
                    context.use_value(&identifier.node)?
                } else {
                    context.get_value(&identifier.node)?
                };

                if let Some(value) = value_option {
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
            ValueExpression::MapIndex(map_index) => map_index.node.run(context, manage_memory),
            ValueExpression::ListIndex(list_index) => list_index.node.run(context, manage_memory),
            ValueExpression::Logic(logic) => logic.node.run(context, manage_memory),
            ValueExpression::Math(math) => math.node.run(context, manage_memory),
            ValueExpression::Value(value_node) => value_node.node.run(context, manage_memory),
            ValueExpression::BuiltInFunctionCall(built_in_function_call) => {
                built_in_function_call.node.run(context, manage_memory)
            }
        }
    }
}

impl ExpectedType for ValueExpression {
    fn expected_type(&self, _context: &mut Context) -> Result<Type, ValidationError> {
        match self {
            ValueExpression::As(r#as) => r#as.node.expected_type(_context),
            ValueExpression::FunctionCall(function_call) => {
                function_call.node.expected_type(_context)
            }
            ValueExpression::Identifier(identifier) => {
                if let Some(r#type) = _context.get_type(&identifier.node)? {
                    Ok(r#type)
                } else {
                    Err(ValidationError::VariableNotFound {
                        identifier: identifier.node.clone(),
                        position: identifier.position,
                    })
                }
            }
            ValueExpression::MapIndex(map_index) => map_index.node.expected_type(_context),
            ValueExpression::ListIndex(list_index) => list_index.node.expected_type(_context),
            ValueExpression::Logic(logic) => logic.node.expected_type(_context),
            ValueExpression::Math(math) => math.node.expected_type(_context),
            ValueExpression::Value(value_node) => value_node.node.expected_type(_context),
            ValueExpression::BuiltInFunctionCall(built_in_function_call) => {
                built_in_function_call.node.expected_type(_context)
            }
        }
    }
}
