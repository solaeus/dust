use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{tool::ToolCall, AbstractTree, Error, Identifier, Result, Value, VariableMap};

use super::{function_call::FunctionCall, logic::Logic, math::Math};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Expression {
    Identifier(Identifier),
    Value(Value),
    Math(Box<Math>),
    Logic(Box<Logic>),
    FunctionCall(FunctionCall),
    ToolCall(Box<ToolCall>),
}

impl AbstractTree for Expression {
    fn from_syntax_node(node: Node, source: &str) -> Result<Self> {
        debug_assert_eq!("expression", node.kind());

        let child = node.child(0).unwrap();

        let expression = match child.kind() {
            "value" => Expression::Value(Value::from_syntax_node(child, source)?),
            "identifier" => Self::Identifier(Identifier::from_syntax_node(child, source)?),
            "math" => Expression::Math(Box::new(Math::from_syntax_node(child, source)?)),
            "logic" => Expression::Logic(Box::new(Logic::from_syntax_node(child, source)?)),
            "function_call" => {
                Expression::FunctionCall(FunctionCall::from_syntax_node(child, source)?)
            }
            "tool_call" => {
                Expression::ToolCall(Box::new(ToolCall::from_syntax_node(child, source)?))
            }
            _ => {
                return Err(Error::UnexpectedSyntax {
                    expected: "value, identifier, math or function_call",
                    actual: child.kind(),
                    location: child.start_position(),
                    relevant_source: source[node.byte_range()].to_string(),
                })
            }
        };

        Ok(expression)
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        match self {
            Expression::Value(value) => Ok(value.clone()),
            Expression::Identifier(identifier) => identifier.run(context),
            Expression::Math(math) => math.run(context),
            Expression::Logic(logic) => logic.run(context),
            Expression::FunctionCall(function_call) => function_call.run(context),
            Expression::ToolCall(tool_call) => tool_call.run(context),
        }
    }
}
