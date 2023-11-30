use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    AbstractTree, Assignment, Block, Error, Expression, For, IfElse, IndexAssignment, Map, Match,
    Result, Type, Use, Value, While,
};

/// Abstract representation of a statement.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Statement {
    Assignment(Box<Assignment>),
    Return(Expression),
    Expression(Expression),
    IfElse(Box<IfElse>),
    Match(Match),
    While(Box<While>),
    Block(Box<Block>),
    For(Box<For>),
    Use(Use),
    IndexAssignment(Box<IndexAssignment>),
}

impl AbstractTree for Statement {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "statement", node)?;

        let child = node.child(0).unwrap();

        match child.kind() {
            "assignment" => Ok(Statement::Assignment(Box::new(
                Assignment::from_syntax_node(source, child, context)?,
            ))),
            "return" => {
                let expression_node = child.child(1).unwrap();

                Ok(Statement::Return(Expression::from_syntax_node(source, expression_node, context)?))
            },
            "expression" => Ok(Self::Expression(Expression::from_syntax_node(
                source, child, context
            )?)),
            "if_else" => Ok(Statement::IfElse(Box::new(IfElse::from_syntax_node(
                source, child, context
            )?))),
            "tool" => Ok(Statement::IfElse(Box::new(IfElse::from_syntax_node(
                source, child, context
            )?))),
            "while" => Ok(Statement::While(Box::new(While::from_syntax_node(
                source, child, context
            )?))),
            "block" => Ok(Statement::Block(Box::new(Block::from_syntax_node(
                source, child, context
            )?))),
            "for" => Ok(Statement::For(Box::new(For::from_syntax_node(
                source, child, context
            )?))),
            "use" => Ok(Statement::Use(Use::from_syntax_node(source, child, context)?)),
            "index_assignment" => Ok(Statement::IndexAssignment(Box::new(IndexAssignment::from_syntax_node(
                source, child, context
            )?))),
            _ => Err(Error::UnexpectedSyntaxNode {
                expected: "assignment, expression, if...else, while, for, transform, filter, tool, async, find, remove, select, insert, index_assignment or yield",
                actual: child.kind(),
                location: child.start_position(),
                relevant_source: source[child.byte_range()].to_string(),
            }),
        }
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        match self {
            Statement::Assignment(assignment) => assignment.run(source, context),
            Statement::Return(expression) => expression.run(source, context),
            Statement::Expression(expression) => expression.run(source, context),
            Statement::IfElse(if_else) => if_else.run(source, context),
            Statement::Match(r#match) => r#match.run(source, context),
            Statement::While(r#while) => r#while.run(source, context),
            Statement::Block(block) => block.run(source, context),
            Statement::For(r#for) => r#for.run(source, context),
            Statement::Use(run) => run.run(source, context),
            Statement::IndexAssignment(index_assignment) => index_assignment.run(source, context),
        }
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        match self {
            Statement::Assignment(assignment) => assignment.expected_type(context),
            Statement::Return(expression) => expression.expected_type(context),
            Statement::Expression(expression) => expression.expected_type(context),
            Statement::IfElse(if_else) => if_else.expected_type(context),
            Statement::Match(r#match) => r#match.expected_type(context),
            Statement::While(r#while) => r#while.expected_type(context),
            Statement::Block(block) => block.expected_type(context),
            Statement::For(r#for) => r#for.expected_type(context),
            Statement::Use(r#use) => r#use.expected_type(context),
            Statement::IndexAssignment(index_assignment) => index_assignment.expected_type(context),
        }
    }
}
