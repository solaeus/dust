use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Assignment, Block, Error, Expression, For, Format, IfElse, IndexAssignment, Map,
    Match, SyntaxNode, Type, Value, While,
};

/// Abstract representation of a statement.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Statement {
    Assignment(Box<Assignment>),
    Expression(Expression),
    IfElse(Box<IfElse>),
    Match(Match),
    While(Box<While>),
    Block(Box<Block>),
    Return(Box<Statement>),
    For(Box<For>),
    IndexAssignment(Box<IndexAssignment>),
}

impl AbstractTree for Statement {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self, SyntaxError> {
        Error::expect_syntax_node(source, "statement", node)?;

        let child = node.child(0).unwrap();

        match child.kind() {
            "assignment" => Ok(Statement::Assignment(Box::new(
                Assignment::from_syntax(child, source, context)?,
            ))),
            "expression" => Ok(Statement::Expression(Expression::from_syntax(
                child, source, context,
            )?)),
            "if_else" => Ok(Statement::IfElse(Box::new(IfElse::from_syntax(
                child, source, context,
            )?))),
            "while" => Ok(Statement::While(Box::new(While::from_syntax(
                child, source, context,
            )?))),
            "block" => Ok(Statement::Block(Box::new(Block::from_syntax(
                child, source, context,
            )?))),
            "for" => Ok(Statement::For(Box::new(For::from_syntax(
                child, source, context,
            )?))),
            "index_assignment" => Ok(Statement::IndexAssignment(Box::new(
                IndexAssignment::from_syntax(child, source, context)?,
            ))),
            "match" => Ok(Statement::Match(Match::from_syntax(
                child, source, context,
            )?)),
            "return" => {
                let statement_node = child.child(1).unwrap();

                Ok(Statement::Return(Box::new(Statement::from_syntax(statement_node, source, context)?)))
            },
            _ => Err(Error::UnexpectedSyntaxNode {
                expected:
                    "assignment, index assignment, expression, block, return, if...else, while, for or match".to_string(),
                actual: child.kind().to_string(),
                location: child.start_position(),
                relevant_source: source[child.byte_range()].to_string(),
            }),
        }
    }

    fn expected_type(&self, context: &Map) -> Result<Type, ValidationError> {
        match self {
            Statement::Assignment(assignment) => assignment.expected_type(context),
            Statement::Expression(expression) => expression.expected_type(context),
            Statement::IfElse(if_else) => if_else.expected_type(context),
            Statement::Match(r#match) => r#match.expected_type(context),
            Statement::While(r#while) => r#while.expected_type(context),
            Statement::Block(block) => block.expected_type(context),
            Statement::For(r#for) => r#for.expected_type(context),
            Statement::IndexAssignment(index_assignment) => index_assignment.expected_type(context),
            Statement::Return(statement) => statement.expected_type(context),
        }
    }

    fn check_type(&self, _source: &str, _context: &Map) -> Result<(), ValidationError> {
        match self {
            Statement::Assignment(assignment) => assignment.check_type(_source, _context),
            Statement::Expression(expression) => expression.check_type(_source, _context),
            Statement::IfElse(if_else) => if_else.check_type(_source, _context),
            Statement::Match(r#match) => r#match.check_type(_source, _context),
            Statement::While(r#while) => r#while.check_type(_source, _context),
            Statement::Block(block) => block.check_type(_source, _context),
            Statement::For(r#for) => r#for.check_type(_source, _context),
            Statement::IndexAssignment(index_assignment) => {
                index_assignment.check_type(_source, _context)
            }
            Statement::Return(statement) => statement.check_type(_source, _context),
        }
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value, RuntimeError> {
        match self {
            Statement::Assignment(assignment) => assignment.run(source, context),
            Statement::Expression(expression) => expression.run(source, context),
            Statement::IfElse(if_else) => if_else.run(source, context),
            Statement::Match(r#match) => r#match.run(source, context),
            Statement::While(r#while) => r#while.run(source, context),
            Statement::Block(block) => block.run(source, context),
            Statement::For(r#for) => r#for.run(source, context),
            Statement::IndexAssignment(index_assignment) => index_assignment.run(source, context),
            Statement::Return(statement) => statement.run(source, context),
        }
    }
}

impl Format for Statement {
    fn format(&self, output: &mut String, indent_level: u8) {
        Statement::indent(output, indent_level);

        match self {
            Statement::Assignment(assignment) => assignment.format(output, indent_level),
            Statement::Expression(expression) => expression.format(output, indent_level),
            Statement::IfElse(if_else) => if_else.format(output, indent_level),
            Statement::Match(r#match) => r#match.format(output, indent_level),
            Statement::While(r#while) => r#while.format(output, indent_level),
            Statement::Block(block) => block.format(output, indent_level),
            Statement::For(r#for) => r#for.format(output, indent_level),
            Statement::IndexAssignment(index_assignment) => {
                index_assignment.format(output, indent_level)
            }
            Statement::Return(statement) => statement.format(output, indent_level),
        }
    }
}
