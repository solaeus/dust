use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    r#async::Async, r#while::While, AbstractTree, Assignment, Error, Expression, IfElse, Match,
    Result, Value, VariableMap,
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
    Run(Box<Async>),
}

impl AbstractTree for Statement {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        debug_assert_eq!("statement", node.kind());

        let child = node.child(0).unwrap();

        match child.kind() {
            "assignment" => Ok(Statement::Assignment(Box::new(
                Assignment::from_syntax_node(source, child)?,
            ))),
            "expression" => Ok(Self::Expression(Expression::from_syntax_node(
                source, child,
            )?)),
            "if_else" => Ok(Statement::IfElse(Box::new(IfElse::from_syntax_node(
                source, child,
            )?))),
            "tool" => Ok(Statement::IfElse(Box::new(IfElse::from_syntax_node(
                source, child,
            )?))),
            "while" => Ok(Statement::While(Box::new(While::from_syntax_node(
                source, child,
            )?))),
            "async" => Ok(Statement::Run(Box::new(Async::from_syntax_node(
                source, child,
            )?))),
            _ => Err(Error::UnexpectedSyntaxNode {
                expected: "assignment, expression, if...else, while, tool or async",
                actual: child.kind(),
                location: child.start_position(),
                relevant_source: source[child.byte_range()].to_string(),
            }),
        }
    }

    fn run(&self, source: &str, context: &mut VariableMap) -> Result<Value> {
        match self {
            Statement::Assignment(assignment) => assignment.run(source, context),
            Statement::Expression(expression) => expression.run(source, context),
            Statement::IfElse(if_else) => if_else.run(source, context),
            Statement::Match(r#match) => r#match.run(source, context),
            Statement::While(r#while) => r#while.run(source, context),
            Statement::Run(run) => run.run(source, context),
        }
    }
}
