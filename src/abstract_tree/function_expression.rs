use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    AbstractTree, Error, FunctionCall, Identifier, Index, Map, Result, Type, Value, ValueNode,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum FunctionExpression {
    Identifier(Identifier),
    FunctionCall(Box<FunctionCall>),
    Value(ValueNode),
    Index(Index),
}

impl AbstractTree for FunctionExpression {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "function_expression", node)?;

        let child = node.child(0).unwrap();

        let function_expression = match child.kind() {
            "identifier" => FunctionExpression::Identifier(Identifier::from_syntax_node(
                source, child, context,
            )?),
            "function_call" => FunctionExpression::FunctionCall(Box::new(
                FunctionCall::from_syntax_node(source, child, context)?,
            )),
            "value" => {
                FunctionExpression::Value(ValueNode::from_syntax_node(source, child, context)?)
            }
            "index" => FunctionExpression::Index(Index::from_syntax_node(source, child, context)?),
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "identifier, function call, value or index",
                    actual: child.kind(),
                    location: child.start_position(),
                    relevant_source: source[child.byte_range()].to_string(),
                })
            }
        };

        Ok(function_expression)
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        match self {
            FunctionExpression::Identifier(identifier) => identifier.run(source, context),
            FunctionExpression::FunctionCall(function_call) => function_call.run(source, context),
            FunctionExpression::Value(value_node) => value_node.run(source, context),
            FunctionExpression::Index(index) => index.run(source, context),
        }
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        match self {
            FunctionExpression::Identifier(identifier) => identifier.expected_type(context),
            FunctionExpression::FunctionCall(function_call) => function_call.expected_type(context),
            FunctionExpression::Value(value_node) => value_node.expected_type(context),
            FunctionExpression::Index(index) => index.expected_type(context),
        }
    }
}
