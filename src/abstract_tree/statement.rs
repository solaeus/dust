use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Assignment, Block, Context, Expression, For, Format, IfElse, IndexAssignment,
    Match, SyntaxNode, Type, TypeDefinition, Value, While,
};

/// Abstract representation of a statement.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Statement {
    is_return: bool,
    statement_kind: StatementKind,
}

impl Statement {
    pub fn is_return(&self) -> bool {
        self.is_return
    }
}

impl AbstractTree for Statement {
    fn from_syntax(
        node: SyntaxNode,
        source: &str,
        _context: &Context,
    ) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node("statement", node)?;

        let first_child = node.child(0).unwrap();
        let mut is_return = first_child.kind() == "return";
        let child = if is_return {
            node.child(1).unwrap()
        } else {
            first_child
        };

        let statement_kind = StatementKind::from_syntax(child, source, _context)?;

        if let StatementKind::Block(block) = &statement_kind {
            if block.contains_return() {
                is_return = true;
            }
        };

        Ok(Statement {
            is_return,
            statement_kind,
        })
    }

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        self.statement_kind.expected_type(_context)
    }

    fn validate(&self, _source: &str, _context: &Context) -> Result<(), ValidationError> {
        self.statement_kind.validate(_source, _context)
    }

    fn run(&self, _source: &str, _context: &Context) -> Result<Value, RuntimeError> {
        self.statement_kind.run(_source, _context)
    }
}

impl Format for Statement {
    fn format(&self, _output: &mut String, _indent_level: u8) {
        self.statement_kind.format(_output, _indent_level)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
enum StatementKind {
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

impl AbstractTree for StatementKind {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node("statement_kind", node)?;

        let child = node.child(0).unwrap();

        match child.kind() {
            "assignment" => Ok(StatementKind::Assignment(Box::new(
                Assignment::from_syntax(child, source, context)?,
            ))),
            "expression" => Ok(StatementKind::Expression(Expression::from_syntax(
                child, source, context,
            )?)),
            "if_else" => Ok(StatementKind::IfElse(Box::new(IfElse::from_syntax(
                child, source, context,
            )?))),
            "while" => Ok(StatementKind::While(Box::new(While::from_syntax(
                child, source, context,
            )?))),
            "block" => Ok(StatementKind::Block(Box::new(Block::from_syntax(
                child, source, context,
            )?))),
            "for" => Ok(StatementKind::For(Box::new(For::from_syntax(
                child, source, context,
            )?))),
            "index_assignment" => Ok(StatementKind::IndexAssignment(Box::new(
                IndexAssignment::from_syntax(child, source, context)?,
            ))),
            "match" => Ok(StatementKind::Match(Match::from_syntax(
                child, source, context,
            )?)),
            "type_definition" => Ok(StatementKind::TypeDefinition(TypeDefinition::from_syntax(
                child, source, context
            )?)),
            _ => Err(SyntaxError::UnexpectedSyntaxNode {
                expected:
                    "assignment, index assignment, expression, type_definition, block, return, if...else, while, for or match".to_string(),
                actual: child.kind().to_string(),
                position: node.range().into(),
            }),
        }
    }

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match self {
            StatementKind::Assignment(assignment) => assignment.expected_type(_context),
            StatementKind::Expression(expression) => expression.expected_type(_context),
            StatementKind::IfElse(if_else) => if_else.expected_type(_context),
            StatementKind::Match(r#match) => r#match.expected_type(_context),
            StatementKind::While(r#while) => r#while.expected_type(_context),
            StatementKind::Block(block) => block.expected_type(_context),
            StatementKind::For(r#for) => r#for.expected_type(_context),
            StatementKind::IndexAssignment(index_assignment) => {
                index_assignment.expected_type(_context)
            }
            StatementKind::TypeDefinition(type_definition) => {
                type_definition.expected_type(_context)
            }
        }
    }

    fn validate(&self, _source: &str, _context: &Context) -> Result<(), ValidationError> {
        match self {
            StatementKind::Assignment(assignment) => assignment.validate(_source, _context),
            StatementKind::Expression(expression) => expression.validate(_source, _context),
            StatementKind::IfElse(if_else) => if_else.validate(_source, _context),
            StatementKind::Match(r#match) => r#match.validate(_source, _context),
            StatementKind::While(r#while) => r#while.validate(_source, _context),
            StatementKind::Block(block) => block.validate(_source, _context),
            StatementKind::For(r#for) => r#for.validate(_source, _context),
            StatementKind::IndexAssignment(index_assignment) => {
                index_assignment.validate(_source, _context)
            }
            StatementKind::TypeDefinition(type_definition) => {
                type_definition.validate(_source, _context)
            }
        }
    }

    fn run(&self, _source: &str, _context: &Context) -> Result<Value, RuntimeError> {
        match self {
            StatementKind::Assignment(assignment) => assignment.run(_source, _context),
            StatementKind::Expression(expression) => expression.run(_source, _context),
            StatementKind::IfElse(if_else) => if_else.run(_source, _context),
            StatementKind::Match(r#match) => r#match.run(_source, _context),
            StatementKind::While(r#while) => r#while.run(_source, _context),
            StatementKind::Block(block) => block.run(_source, _context),
            StatementKind::For(r#for) => r#for.run(_source, _context),
            StatementKind::IndexAssignment(index_assignment) => {
                index_assignment.run(_source, _context)
            }
            StatementKind::TypeDefinition(type_definition) => {
                type_definition.run(_source, _context)
            }
        }
    }
}

impl Format for StatementKind {
    fn format(&self, output: &mut String, indent_level: u8) {
        StatementKind::indent(output, indent_level);

        match self {
            StatementKind::Assignment(assignment) => assignment.format(output, indent_level),
            StatementKind::Expression(expression) => expression.format(output, indent_level),
            StatementKind::IfElse(if_else) => if_else.format(output, indent_level),
            StatementKind::Match(r#match) => r#match.format(output, indent_level),
            StatementKind::While(r#while) => r#while.format(output, indent_level),
            StatementKind::Block(block) => block.format(output, indent_level),
            StatementKind::For(r#for) => r#for.format(output, indent_level),
            StatementKind::IndexAssignment(index_assignment) => {
                index_assignment.format(output, indent_level)
            }
            StatementKind::TypeDefinition(type_definition) => {
                type_definition.format(output, indent_level)
            }
        }
    }
}
