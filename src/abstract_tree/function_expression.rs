use serde::{Deserialize, Serialize};

use crate::{
    AbstractTree, Error, Format, FunctionCall, Identifier, Index, Map, Result, SyntaxNode, Type,
    Value, ValueNode, Yield,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum FunctionExpression {
    Identifier(Identifier),
    FunctionCall(Box<FunctionCall>),
    Value(ValueNode),
    Index(Index),
    Yield(Box<Yield>),
}

impl AbstractTree for FunctionExpression {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "function_expression", node)?;

        let first_child = node.child(0).unwrap();
        let child = if first_child.is_named() {
            first_child
        } else {
            node.child(1).unwrap()
        };

        let function_expression = match child.kind() {
            "identifier" => {
                FunctionExpression::Identifier(Identifier::from_syntax(child, source, context)?)
            }

            "function_call" => FunctionExpression::FunctionCall(Box::new(
                FunctionCall::from_syntax(child, source, context)?,
            )),
            "value" => FunctionExpression::Value(ValueNode::from_syntax(child, source, context)?),
            "index" => FunctionExpression::Index(Index::from_syntax(child, source, context)?),
            "yield" => {
                FunctionExpression::Yield(Box::new(Yield::from_syntax(child, source, context)?))
            }
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "identifier, function call, value, index or yield".to_string(),
                    actual: child.kind().to_string(),
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
            FunctionExpression::Yield(r#yield) => r#yield.run(source, context),
        }
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        match self {
            FunctionExpression::Identifier(identifier) => identifier.expected_type(context),
            FunctionExpression::FunctionCall(function_call) => function_call.expected_type(context),
            FunctionExpression::Value(value_node) => value_node.expected_type(context),
            FunctionExpression::Index(index) => index.expected_type(context),
            FunctionExpression::Yield(r#yield) => r#yield.expected_type(context),
        }
    }
}

impl Format for FunctionExpression {
    fn format(&self, output: &mut String, indent_level: u8) {
        match self {
            FunctionExpression::Value(value_node) => value_node.format(output, indent_level),
            FunctionExpression::Identifier(identifier) => identifier.format(output, indent_level),
            FunctionExpression::FunctionCall(function_call) => {
                function_call.format(output, indent_level)
            }
            FunctionExpression::Index(index) => index.format(output, indent_level),
            FunctionExpression::Yield(r#yield) => r#yield.format(output, indent_level),
        }
    }
}
