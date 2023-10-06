use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    tool::Tool, AbstractTree, Assignment, Error, Expression, IfElse, Match, Result, Value,
    VariableMap,
};

/// Abstract representation of a statement.
///
/// Items are either comments, which do nothing, or statements, which can be run
/// to produce a single value or interact with their context.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Statement {
    Assignment(Box<Assignment>),
    Expression(Expression),
    IfElse(Box<IfElse>),
    Match(Match),
    Tool(Tool),
}

impl AbstractTree for Statement {
    fn from_syntax_node(node: Node, source: &str) -> Result<Self> {
        debug_assert_eq!("statement", node.kind());

        let child = node.child(0).unwrap();

        match child.kind() {
            "assignment" => Ok(Statement::Assignment(Box::new(
                Assignment::from_syntax_node(child, source)?,
            ))),
            "expression" => Ok(Self::Expression(Expression::from_syntax_node(
                child, source,
            )?)),
            "if_else" => Ok(Statement::IfElse(Box::new(IfElse::from_syntax_node(
                child, source,
            )?))),
            "tool" => Ok(Statement::IfElse(Box::new(IfElse::from_syntax_node(
                child, source,
            )?))),
            _ => Err(Error::UnexpectedSyntax {
                expected: "assignment, expression, if...else or tool",
                actual: child.kind(),
                location: child.start_position(),
                relevant_source: source[node.byte_range()].to_string(),
            }),
        }
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        match self {
            Statement::Assignment(assignment) => assignment.run(context),
            Statement::Expression(expression) => expression.run(context),
            Statement::IfElse(if_else) => if_else.run(context),
            Statement::Match(r#match) => r#match.run(context),
            Statement::Tool(tool) => tool.run(context),
        }
    }
}
