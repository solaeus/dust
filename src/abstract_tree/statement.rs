use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Assignment, Block, Context, Expression, For, Format, IfElse, IndexAssignment,
    Match, SyntaxNode, Type, TypeDefinition, Value, While,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Statement {
    is_return: bool,
    statement_inner: StatementInner,
}

impl AbstractTree for Statement {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        todo!()
    }

    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError> {
        todo!()
    }

    fn validate(&self, source: &str, context: &Context) -> Result<(), ValidationError> {
        todo!()
    }

    fn run(&self, source: &str, context: &Context) -> Result<Value, RuntimeError> {
        todo!()
    }
}

impl Format for Statement {
    fn format(&self, output: &mut String, indent_level: u8) {
        todo!()
    }
}

/// Abstract representation of a statement.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum StatementInner {
    Assignment(Box<Assignment>),
    Expression(Expression),
    IfElse(Box<IfElse>),
    Match(Match),
    While(Box<While>),
    Block(Box<Block>),
    For(Box<For>),
    IndexAssignment(Box<IndexAssignment>),
    TypeDefinition(TypeDefinition),
}

impl AbstractTree for StatementInner {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(source, "statement", node)?;

        let child = node.child(0).unwrap();

        match child.kind() {
            "assignment" => Ok(StatementInner::Assignment(Box::new(
                Assignment::from_syntax(child, source, context)?,
            ))),
            "expression" => Ok(StatementInner::Expression(Expression::from_syntax(
                child, source, context,
            )?)),
            "if_else" => Ok(StatementInner::IfElse(Box::new(IfElse::from_syntax(
                child, source, context,
            )?))),
            "while" => Ok(StatementInner::While(Box::new(While::from_syntax(
                child, source, context,
            )?))),
            "block" => Ok(StatementInner::Block(Box::new(Block::from_syntax(
                child, source, context,
            )?))),
            "for" => Ok(StatementInner::For(Box::new(For::from_syntax(
                child, source, context,
            )?))),
            "index_assignment" => Ok(StatementInner::IndexAssignment(Box::new(
                IndexAssignment::from_syntax(child, source, context)?,
            ))),
            "match" => Ok(StatementInner::Match(Match::from_syntax(
                child, source, context,
            )?)),
            "type_definition" => Ok(StatementInner::TypeDefinition(TypeDefinition::from_syntax(
                child, source, context
            )?)),
            _ => Err(SyntaxError::UnexpectedSyntaxNode {
                expected:
                    "assignment, index assignment, expression, type_definition, block, return, if...else, while, for or match".to_string(),
                actual: child.kind().to_string(),
                location: child.start_position(),
                relevant_source: source[child.byte_range()].to_string(),
            }),
        }
    }

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match self {
            StatementInner::Assignment(assignment) => assignment.expected_type(_context),
            StatementInner::Expression(expression) => expression.expected_type(_context),
            StatementInner::IfElse(if_else) => if_else.expected_type(_context),
            StatementInner::Match(r#match) => r#match.expected_type(_context),
            StatementInner::While(r#while) => r#while.expected_type(_context),
            StatementInner::Block(block) => block.expected_type(_context),
            StatementInner::For(r#for) => r#for.expected_type(_context),
            StatementInner::IndexAssignment(index_assignment) => {
                index_assignment.expected_type(_context)
            }
            StatementInner::TypeDefinition(type_definition) => {
                type_definition.expected_type(_context)
            }
        }
    }

    fn validate(&self, _source: &str, _context: &Context) -> Result<(), ValidationError> {
        match self {
            StatementInner::Assignment(assignment) => assignment.validate(_source, _context),
            StatementInner::Expression(expression) => expression.validate(_source, _context),
            StatementInner::IfElse(if_else) => if_else.validate(_source, _context),
            StatementInner::Match(r#match) => r#match.validate(_source, _context),
            StatementInner::While(r#while) => r#while.validate(_source, _context),
            StatementInner::Block(block) => block.validate(_source, _context),
            StatementInner::For(r#for) => r#for.validate(_source, _context),
            StatementInner::IndexAssignment(index_assignment) => {
                index_assignment.validate(_source, _context)
            }
            StatementInner::TypeDefinition(type_definition) => {
                type_definition.validate(_source, _context)
            }
        }
    }

    fn run(&self, _source: &str, _context: &Context) -> Result<Value, RuntimeError> {
        match self {
            StatementInner::Assignment(assignment) => assignment.run(_source, _context),
            StatementInner::Expression(expression) => expression.run(_source, _context),
            StatementInner::IfElse(if_else) => if_else.run(_source, _context),
            StatementInner::Match(r#match) => r#match.run(_source, _context),
            StatementInner::While(r#while) => r#while.run(_source, _context),
            StatementInner::Block(block) => block.run(_source, _context),
            StatementInner::For(r#for) => r#for.run(_source, _context),
            StatementInner::IndexAssignment(index_assignment) => {
                index_assignment.run(_source, _context)
            }
            StatementInner::TypeDefinition(type_definition) => {
                type_definition.run(_source, _context)
            }
        }
    }
}

impl Format for StatementInner {
    fn format(&self, output: &mut String, indent_level: u8) {
        StatementInner::indent(output, indent_level);

        match self {
            StatementInner::Assignment(assignment) => assignment.format(output, indent_level),
            StatementInner::Expression(expression) => expression.format(output, indent_level),
            StatementInner::IfElse(if_else) => if_else.format(output, indent_level),
            StatementInner::Match(r#match) => r#match.format(output, indent_level),
            StatementInner::While(r#while) => r#while.format(output, indent_level),
            StatementInner::Block(block) => block.format(output, indent_level),
            StatementInner::For(r#for) => r#for.format(output, indent_level),
            StatementInner::IndexAssignment(index_assignment) => {
                index_assignment.format(output, indent_level)
            }
            StatementInner::TypeDefinition(type_definition) => {
                type_definition.format(output, indent_level)
            }
        }
    }
}
