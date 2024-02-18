use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    value_node::ValueNode,
    AbstractTree, Context, Format, FunctionCall, Identifier, Index, SyntaxNode, Type, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum IndexExpression {
    Value(ValueNode),
    Identifier(Identifier),
    Index(Box<Index>),
    FunctionCall(Box<FunctionCall>),
}

impl AbstractTree for IndexExpression {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node("index_expression", node)?;

        let first_child = node.child(0).unwrap();
        let child = if first_child.is_named() {
            first_child
        } else {
            node.child(1).unwrap()
        };

        let abstract_node = match child.kind() {
            "value" => IndexExpression::Value(ValueNode::from_syntax(child, source, context)?),
            "identifier" => {
                IndexExpression::Identifier(Identifier::from_syntax(child, source, context)?)
            }
            "index" => {
                IndexExpression::Index(Box::new(Index::from_syntax(child, source, context)?))
            }
            "function_call" => IndexExpression::FunctionCall(Box::new(FunctionCall::from_syntax(
                child, source, context,
            )?)),
            _ => {
                return Err(SyntaxError::UnexpectedSyntaxNode {
                    expected: "value, identifier, index or function call".to_string(),
                    actual: child.kind().to_string(),
                    position: node.range().into(),
                })
            }
        };

        Ok(abstract_node)
    }

    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError> {
        match self {
            IndexExpression::Value(value_node) => value_node.expected_type(context),
            IndexExpression::Identifier(identifier) => identifier.expected_type(context),
            IndexExpression::Index(index) => index.expected_type(context),
            IndexExpression::FunctionCall(function_call) => function_call.expected_type(context),
        }
    }

    fn validate(&self, _source: &str, context: &Context) -> Result<(), ValidationError> {
        match self {
            IndexExpression::Value(value_node) => value_node.validate(_source, context),
            IndexExpression::Identifier(identifier) => {
                context.add_allowance(identifier)?;

                Ok(())
            }
            IndexExpression::Index(index) => index.validate(_source, context),
            IndexExpression::FunctionCall(function_call) => {
                function_call.validate(_source, context)
            }
        }
    }

    fn run(&self, source: &str, context: &Context) -> Result<Value, RuntimeError> {
        match self {
            IndexExpression::Value(value_node) => value_node.run(source, context),
            IndexExpression::Identifier(identifier) => identifier.run(source, context),
            IndexExpression::Index(index) => index.run(source, context),
            IndexExpression::FunctionCall(function_call) => function_call.run(source, context),
        }
    }
}

impl Format for IndexExpression {
    fn format(&self, output: &mut String, indent_level: u8) {
        match self {
            IndexExpression::Value(value_node) => {
                value_node.format(output, indent_level);
            }
            IndexExpression::Identifier(identifier) => identifier.format(output, indent_level),
            IndexExpression::FunctionCall(function_call) => {
                function_call.format(output, indent_level)
            }
            IndexExpression::Index(index) => index.format(output, indent_level),
        }
    }
}
