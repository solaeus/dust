use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    value_node::ValueNode, AbstractTree, BuiltInFunction, Error, Identifier, Index, Map, Result,
    Value,
};

use super::{function_call::FunctionCall, logic::Logic, math::Math};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Expression {
    Value(ValueNode),
    Identifier(Identifier),
    Index(Box<Index>),
    Math(Box<Math>),
    Logic(Box<Logic>),
    FunctionCall(FunctionCall),
    Tool(Box<BuiltInFunction>),
}

impl AbstractTree for Expression {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        Error::expect_syntax_node(source, "expression", node)?;

        for index in 0..node.child_count() {
            let child = node.child(index).unwrap();
            let expression = match child.kind() {
                "value" => Expression::Value(ValueNode::from_syntax_node(source, child)?),
                "identifier" => {
                    Expression::Identifier(Identifier::from_syntax_node(source, child)?)
                }
                "index" => Expression::Index(Box::new(Index::from_syntax_node(source, child)?)),
                "math" => Expression::Math(Box::new(Math::from_syntax_node(source, child)?)),
                "logic" => Expression::Logic(Box::new(Logic::from_syntax_node(source, child)?)),
                "function_call" => {
                    Expression::FunctionCall(FunctionCall::from_syntax_node(source, child)?)
                }
                "tool" => {
                    Expression::Tool(Box::new(BuiltInFunction::from_syntax_node(source, child)?))
                }
                _ => continue,
            };

            return Ok(expression);
        }

        let child = node.child(0).unwrap();

        Err(Error::UnexpectedSyntaxNode {
            expected: "value, identifier, sublist, index, math or function_call",
            actual: child.kind(),
            location: child.start_position(),
            relevant_source: source[child.byte_range()].to_string(),
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        match self {
            Expression::Value(value_node) => value_node.run(source, context),
            Expression::Identifier(identifier) => identifier.run(source, context),
            Expression::Math(math) => math.run(source, context),
            Expression::Logic(logic) => logic.run(source, context),
            Expression::FunctionCall(function_call) => function_call.run(source, context),
            Expression::Tool(tool) => tool.run(source, context),
            Expression::Index(index) => index.run(source, context),
        }
    }
}
