use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    value_node::ValueNode, AbstractTree, Error, Identifier, Index, Map, Result, Type, Value, Yield,
};

use super::{function_call::FunctionCall, logic::Logic, math::Math};

/// Abstract representation of an expression statement.
///
/// Unlike statements, which can involve complex logic, an expression is
/// expected to evaluate to a value. However, an expression can still contain
/// nested statements and may evaluate to an empty value.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Expression {
    Value(ValueNode),
    Identifier(Identifier),
    Index(Box<Index>),
    Math(Box<Math>),
    Logic(Box<Logic>),
    FunctionCall(Box<FunctionCall>),
    Yield(Box<Yield>),
}

impl AbstractTree for Expression {
    fn from_syntax_node(source: &str, node: Node, context: &mut Map) -> Result<Self> {
        Error::expect_syntax_node(source, "expression", node)?;

        let child = if node.child(0).unwrap().is_named() {
            node.child(0).unwrap()
        } else {
            node.child(1).unwrap()
        };

        let expression = match child.kind() {
            "value" => Expression::Value(ValueNode::from_syntax_node(source, child, context)?),
            "identifier" => {
                Expression::Identifier(Identifier::from_syntax_node(source, child, context)?)
            }
            "index" => {
                Expression::Index(Box::new(Index::from_syntax_node(source, child, context)?))
            }
            "math" => Expression::Math(Box::new(Math::from_syntax_node(source, child, context)?)),
            "logic" => {
                Expression::Logic(Box::new(Logic::from_syntax_node(source, child, context)?))
            }
            "function_call" => Expression::FunctionCall(Box::new(FunctionCall::from_syntax_node(
                source, child, context,
            )?)),
            "yield" => {
                Expression::Yield(Box::new(Yield::from_syntax_node(source, child, context)?))
            }
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "value_node, identifier, index, math, logic, function_call or yield"
                        .to_string(),
                    actual: child.kind().to_string(),
                    location: child.start_position(),
                    relevant_source: source[child.byte_range()].to_string(),
                })
            }
        };

        Ok(expression)
    }

    fn run(&self, _source: &str, _context: &mut Map) -> Result<Value> {
        match self {
            Expression::Value(value_node) => value_node.run(_source, _context),
            Expression::Identifier(identifier) => identifier.run(_source, _context),
            Expression::Math(math) => math.run(_source, _context),
            Expression::Logic(logic) => logic.run(_source, _context),
            Expression::FunctionCall(function_call) => function_call.run(_source, _context),
            Expression::Index(index) => index.run(_source, _context),
            Expression::Yield(r#yield) => r#yield.run(_source, _context),
        }
    }

    fn check_type(&self, _source: &str, _context: &Map) -> Result<()> {
        match self {
            Expression::Value(value_node) => value_node.check_type(_source, _context),
            Expression::Identifier(identifier) => identifier.check_type(_source, _context),
            Expression::Math(math) => math.check_type(_source, _context),
            Expression::Logic(logic) => logic.check_type(_source, _context),
            Expression::FunctionCall(function_call) => function_call.check_type(_source, _context),
            Expression::Index(index) => index.check_type(_source, _context),
            Expression::Yield(r#yield) => r#yield.check_type(_source, _context),
        }
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        match self {
            Expression::Value(value_node) => value_node.expected_type(_context),
            Expression::Identifier(identifier) => identifier.expected_type(_context),
            Expression::Math(math) => math.expected_type(_context),
            Expression::Logic(logic) => logic.expected_type(_context),
            Expression::FunctionCall(function_call) => function_call.expected_type(_context),
            Expression::Index(index) => index.expected_type(_context),
            Expression::Yield(r#yield) => r#yield.expected_type(_context),
        }
    }
}
