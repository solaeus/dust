use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    value_node::ValueNode,
    AbstractTree, As, Command, Context, Format, FunctionCall, Identifier, Index, Logic, Math,
    SyntaxNode, Type, Value,
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
    Command(Command),
    As(Box<As>),
}

impl AbstractTree for Expression {
    fn from_syntax(
        node: SyntaxNode,
        source: &str,
        _context: &Context,
    ) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node("expression", node)?;

        let child = if node.child(0).unwrap().is_named() {
            node.child(0).unwrap()
        } else {
            node.child(1).unwrap()
        };

        let expression = match child.kind() {
            "as" => Expression::As(Box::new(As::from_syntax(child, source, _context)?)),
            "value" => Expression::Value(ValueNode::from_syntax(child, source, _context)?),
            "identifier" => {
                Expression::Identifier(Identifier::from_syntax(child, source, _context)?)
            }
            "index" => Expression::Index(Box::new(Index::from_syntax(child, source, _context)?)),
            "math" => Expression::Math(Box::new(Math::from_syntax(child, source, _context)?)),
            "logic" => Expression::Logic(Box::new(Logic::from_syntax(child, source, _context)?)),
            "function_call" => Expression::FunctionCall(Box::new(FunctionCall::from_syntax(
                child, source, _context,
            )?)),
            "command" => Expression::Command(Command::from_syntax(child, source, _context)?),
            _ => {
                return Err(SyntaxError::UnexpectedSyntaxNode {
                    expected: "value, identifier, index, math, logic, function call, as or command"
                        .to_string(),
                    actual: child.kind().to_string(),
                    position: node.range().into(),
                })
            }
        };

        Ok(expression)
    }

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match self {
            Expression::Value(value_node) => value_node.expected_type(_context),
            Expression::Identifier(identifier) => identifier.expected_type(_context),
            Expression::Math(math) => math.expected_type(_context),
            Expression::Logic(logic) => logic.expected_type(_context),
            Expression::FunctionCall(function_call) => function_call.expected_type(_context),
            Expression::Index(index) => index.expected_type(_context),
            Expression::Command(command) => command.expected_type(_context),
            Expression::As(r#as) => r#as.expected_type(_context),
        }
    }

    fn validate(&self, _source: &str, _context: &Context) -> Result<(), ValidationError> {
        match self {
            Expression::Value(value_node) => value_node.validate(_source, _context),
            Expression::Identifier(identifier) => identifier.validate(_source, _context),
            Expression::Math(math) => math.validate(_source, _context),
            Expression::Logic(logic) => logic.validate(_source, _context),
            Expression::FunctionCall(function_call) => function_call.validate(_source, _context),
            Expression::Index(index) => index.validate(_source, _context),
            Expression::Command(command) => command.validate(_source, _context),
            Expression::As(r#as) => r#as.validate(_source, _context),
        }
    }

    fn run(&self, _source: &str, _context: &Context) -> Result<Value, RuntimeError> {
        match self {
            Expression::Value(value_node) => value_node.run(_source, _context),
            Expression::Identifier(identifier) => identifier.run(_source, _context),
            Expression::Math(math) => math.run(_source, _context),
            Expression::Logic(logic) => logic.run(_source, _context),
            Expression::FunctionCall(function_call) => function_call.run(_source, _context),
            Expression::Index(index) => index.run(_source, _context),
            Expression::Command(command) => command.run(_source, _context),
            Expression::As(r#as) => r#as.run(_source, _context),
        }
    }
}

impl Format for Expression {
    fn format(&self, _output: &mut String, _indent_level: u8) {
        match self {
            Expression::Value(value_node) => value_node.format(_output, _indent_level),
            Expression::Identifier(identifier) => identifier.format(_output, _indent_level),
            Expression::Math(math) => math.format(_output, _indent_level),
            Expression::Logic(logic) => logic.format(_output, _indent_level),
            Expression::FunctionCall(function_call) => function_call.format(_output, _indent_level),
            Expression::Index(index) => index.format(_output, _indent_level),
            Expression::Command(command) => command.format(_output, _indent_level),
            Expression::As(r#as) => r#as.format(_output, _indent_level),
        }
    }
}
