use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
};

use super::{
    As, BuiltInFunctionCall, Evaluate, Evaluation, ExpectedType, FunctionCall, ListIndex, Logic,
    MapIndex, Math, SourcePosition, Type, ValueNode, WithPosition,
};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Expression {
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

impl Expression {
    pub fn position(&self) -> SourcePosition {
        match self {
            Expression::As(inner) => inner.position,
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

impl Evaluate for Expression {
    fn validate(&self, context: &mut Context, manage_memory: bool) -> Result<(), ValidationError> {
        match self {
            Expression::As(r#as) => r#as.node.validate(context, manage_memory),
            Expression::BuiltInFunctionCall(built_in_function_call) => {
                built_in_function_call.node.validate(context, manage_memory)
            }
            Expression::FunctionCall(function_call) => {
                function_call.node.validate(context, manage_memory)
            }
            Expression::Identifier(identifier) => {
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
            Expression::MapIndex(map_index) => map_index.node.validate(context, manage_memory),
            Expression::ListIndex(list_index) => list_index.node.validate(context, manage_memory),
            Expression::Logic(logic) => logic.node.validate(context, manage_memory),
            Expression::Math(math) => math.node.validate(context, manage_memory),
            Expression::Value(value_node) => value_node.node.validate(context, manage_memory),
        }
    }

    fn evaluate(
        self,
        context: &mut Context,
        manage_memory: bool,
    ) -> Result<Evaluation, RuntimeError> {
        match self {
            Expression::As(r#as) => r#as.node.evaluate(context, manage_memory),
            Expression::FunctionCall(function_call) => {
                function_call.node.evaluate(context, manage_memory)
            }
            Expression::Identifier(identifier) => {
                let value_option = if manage_memory {
                    context.use_value(&identifier.node)?
                } else {
                    context.get_value(&identifier.node)?
                };

                if let Some(value) = value_option {
                    Ok(Evaluation::Return(value))
                } else {
                    Err(RuntimeError::ValidationFailure(
                        ValidationError::VariableNotFound {
                            identifier: identifier.node.clone(),
                            position: identifier.position,
                        },
                    ))
                }
            }
            Expression::MapIndex(map_index) => map_index.node.evaluate(context, manage_memory),
            Expression::ListIndex(list_index) => list_index.node.evaluate(context, manage_memory),
            Expression::Logic(logic) => logic.node.evaluate(context, manage_memory),
            Expression::Math(math) => math.node.evaluate(context, manage_memory),
            Expression::Value(value_node) => value_node.node.evaluate(context, manage_memory),
            Expression::BuiltInFunctionCall(built_in_function_call) => {
                built_in_function_call.node.evaluate(context, manage_memory)
            }
        }
    }
}

impl ExpectedType for Expression {
    fn expected_type(&self, _context: &mut Context) -> Result<Type, ValidationError> {
        match self {
            Expression::As(r#as) => r#as.node.expected_type(_context),
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
            Expression::BuiltInFunctionCall(built_in_function_call) => {
                built_in_function_call.node.expected_type(_context)
            }
        }
    }
}
