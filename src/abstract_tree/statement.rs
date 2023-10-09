use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    r#while::While, AbstractTree, Assignment, Error, Expression, IfElse, Match, Result, Value,
    VariableMap,
};

/// Abstract representation of a statement.
///
/// A statement may evaluate to an Empty value when run. If a Statement is an
/// Expression, it will always return a non-empty value when run.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Statement {
    Assignment(Box<Assignment>),
    Expression(Expression),
    IfElse(Box<IfElse>),
    Match(Match),
    While(Box<While>),
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
            "while" => Ok(Statement::While(Box::new(While::from_syntax_node(
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
            Statement::While(r#while) => r#while.run(context),
        }
    }
}
