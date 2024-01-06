use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    value_node::ValueNode, AbstractTree, Error, FunctionCall, Identifier, Index, Map, Result, Type,
    Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum IndexExpression {
    Value(ValueNode),
    Identifier(Identifier),
    Index(Box<Index>),
    FunctionCall(Box<FunctionCall>),
}

impl AbstractTree for IndexExpression {
    fn from_syntax_node(source: &str, node: tree_sitter::Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "index_expression", node)?;

        let first_child = node.child(0).unwrap();
        let child = if first_child.is_named() {
            first_child
        } else {
            node.child(1).unwrap()
        };

        let abstract_node = match child.kind() {
            "value" => IndexExpression::Value(ValueNode::from_syntax_node(source, child, context)?),
            "identifier" => {
                IndexExpression::Identifier(Identifier::from_syntax_node(source, child, context)?)
            }
            "index" => {
                IndexExpression::Index(Box::new(Index::from_syntax_node(source, child, context)?))
            }
            "function_call" => IndexExpression::FunctionCall(Box::new(
                FunctionCall::from_syntax_node(source, child, context)?,
            )),
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "value, identifier, index or function call".to_string(),
                    actual: child.kind().to_string(),
                    location: child.start_position(),
                    relevant_source: source[child.byte_range()].to_string(),
                })
            }
        };

        Ok(abstract_node)
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        match self {
            IndexExpression::Value(value_node) => value_node.run(source, context),
            IndexExpression::Identifier(identifier) => identifier.run(source, context),
            IndexExpression::Index(index) => index.run(source, context),
            IndexExpression::FunctionCall(function_call) => function_call.run(source, context),
        }
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        match self {
            IndexExpression::Value(value_node) => value_node.expected_type(context),
            IndexExpression::Identifier(identifier) => identifier.expected_type(context),
            IndexExpression::Index(index) => index.expected_type(context),
            IndexExpression::FunctionCall(function_call) => function_call.expected_type(context),
        }
    }
}

impl Display for IndexExpression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            IndexExpression::Value(value_node) => write!(f, "{value_node}"),
            IndexExpression::Identifier(identifier) => write!(f, "{identifier}"),
            IndexExpression::FunctionCall(function_call) => write!(f, "{function_call}"),
            IndexExpression::Index(index) => write!(f, "{index}"),
        }
    }
}
