use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    AbstractTree, Assignment, Async, Error, Expression, Filter, Find, For, IfElse, Insert, Map,
    Match, Remove, Result, Select, Transform, Value, While,
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
    Async(Box<Async>),
    For(Box<For>),
    Transform(Box<Transform>),
    Filter(Box<Filter>),
    Find(Box<Find>),
    Remove(Box<Remove>),
    Select(Box<Select>),
    Insert(Box<Insert>),
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
            "async" => Ok(Statement::Async(Box::new(Async::from_syntax_node(
                source, child,
            )?))),
            "for" => Ok(Statement::For(Box::new(For::from_syntax_node(
                source, child,
            )?))),
            "transform" => Ok(Statement::Transform(Box::new(Transform::from_syntax_node(
                source, child,
            )?))),
            "filter" => Ok(Statement::Filter(Box::new(Filter::from_syntax_node(
                source, child,
            )?))),
            "find" => Ok(Statement::Find(Box::new(Find::from_syntax_node(
                source, child,
            )?))),
            "remove" => Ok(Statement::Remove(Box::new(Remove::from_syntax_node(
                source, child,
            )?))),
            "select" => Ok(Statement::Select(Box::new(Select::from_syntax_node(
                source, child,
            )?))),
            "insert" => Ok(Statement::Insert(Box::new(Insert::from_syntax_node(
                source, child,
            )?))),
            _ => Err(Error::UnexpectedSyntaxNode {
                expected: "assignment, expression, if...else, while, for, transform, filter, tool, async, find, remove, select or insert",
                actual: child.kind(),
                location: child.start_position(),
                relevant_source: source[child.byte_range()].to_string(),
            }),
        }
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        match self {
            Statement::Assignment(assignment) => assignment.run(source, context),
            Statement::Expression(expression) => expression.run(source, context),
            Statement::IfElse(if_else) => if_else.run(source, context),
            Statement::Match(r#match) => r#match.run(source, context),
            Statement::While(r#while) => r#while.run(source, context),
            Statement::Async(run) => run.run(source, context),
            Statement::For(r#for) => r#for.run(source, context),
            Statement::Transform(transform) => transform.run(source, context),
            Statement::Filter(filter) => filter.run(source, context),
            Statement::Find(find) => find.run(source, context),
            Statement::Remove(remove) => remove.run(source, context),
            Statement::Select(select) => select.run(source, context),
            Statement::Insert(insert) => insert.run(source, context),
        }
    }
}
