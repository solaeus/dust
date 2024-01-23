use serde::{Deserialize, Serialize};

use crate::{
    value_node::ValueNode, AbstractTree, Error, Format, FunctionCall, Identifier, Index, Logic,
    Map, Math, New, Result, SyntaxNode, Type, Value, Yield,
};

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
    New(New),
}

impl AbstractTree for Expression {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "expression", node)?;

        let child = if node.child(0).unwrap().is_named() {
            node.child(0).unwrap()
        } else {
            node.child(1).unwrap()
        };

        let expression = match child.kind() {
            "value" => Expression::Value(ValueNode::from_syntax(child, source, context)?),
            "identifier" => {
                Expression::Identifier(Identifier::from_syntax(child, source, context)?)
            }
            "index" => Expression::Index(Box::new(Index::from_syntax(child, source, context)?)),
            "math" => Expression::Math(Box::new(Math::from_syntax(child, source, context)?)),
            "logic" => Expression::Logic(Box::new(Logic::from_syntax(child, source, context)?)),
            "function_call" => Expression::FunctionCall(Box::new(FunctionCall::from_syntax(
                child, source, context,
            )?)),
            "yield" => Expression::Yield(Box::new(Yield::from_syntax(child, source, context)?)),
            "new" => Expression::New(New::from_syntax(child, source, context)?),
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "value, identifier, index, math, logic, function call, new or ->"
                        .to_string(),
                    actual: child.kind().to_string(),
                    location: child.start_position(),
                    relevant_source: source[child.byte_range()].to_string(),
                })
            }
        };

        Ok(expression)
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
            Expression::New(_) => todo!(),
        }
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        match self {
            Expression::Value(value_node) => value_node.run(source, context),
            Expression::Identifier(identifier) => identifier.run(source, context),
            Expression::Math(math) => math.run(source, context),
            Expression::Logic(logic) => logic.run(source, context),
            Expression::FunctionCall(function_call) => function_call.run(source, context),
            Expression::Index(index) => index.run(source, context),
            Expression::Yield(r#yield) => r#yield.run(source, context),
            Expression::New(_) => todo!(),
        }
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        match self {
            Expression::Value(value_node) => value_node.expected_type(context),
            Expression::Identifier(identifier) => identifier.expected_type(context),
            Expression::Math(math) => math.expected_type(context),
            Expression::Logic(logic) => logic.expected_type(context),
            Expression::FunctionCall(function_call) => function_call.expected_type(context),
            Expression::Index(index) => index.expected_type(context),
            Expression::Yield(r#yield) => r#yield.expected_type(context),
            Expression::New(_) => todo!(),
        }
    }
}

impl Format for Expression {
    fn format(&self, output: &mut String, indent_level: u8) {
        match self {
            Expression::Value(value_node) => value_node.format(output, indent_level),
            Expression::Identifier(identifier) => identifier.format(output, indent_level),
            Expression::Math(math) => math.format(output, indent_level),
            Expression::Logic(logic) => logic.format(output, indent_level),
            Expression::FunctionCall(function_call) => function_call.format(output, indent_level),
            Expression::Index(index) => index.format(output, indent_level),
            Expression::Yield(r#yield) => r#yield.format(output, indent_level),
            Expression::New(_) => todo!(),
        }
    }
}
