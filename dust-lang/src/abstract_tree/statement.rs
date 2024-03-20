use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{
    AbstractNode, Action, Assignment, AsyncBlock, Block, Expression, IfElse, Loop,
    StructureDefinition, Type, While,
};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Statement {
    Assignment(Assignment),
    AsyncBlock(AsyncBlock),
    Block(Block),
    Break,
    Expression(Expression),
    IfElse(IfElse),
    Loop(Loop),
    StructureDefinition(StructureDefinition),
    While(While),
}

impl Statement {
    pub fn kind(&self) -> u8 {
        match self {
            Statement::StructureDefinition(_) => 0,
            _ => 1,
        }
    }
}

impl AbstractNode for Statement {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match self {
            Statement::Assignment(assignment) => assignment.expected_type(_context),
            Statement::AsyncBlock(async_block) => async_block.expected_type(_context),
            Statement::Block(block) => block.expected_type(_context),
            Statement::Break => Ok(Type::None),
            Statement::Expression(expression) => expression.expected_type(_context),
            Statement::IfElse(if_else) => if_else.expected_type(_context),
            Statement::Loop(r#loop) => r#loop.expected_type(_context),
            Statement::While(r#while) => r#while.expected_type(_context),
            Statement::StructureDefinition(structure_definition) => {
                structure_definition.expected_type(_context)
            }
        }
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        match self {
            Statement::Assignment(assignment) => assignment.validate(_context),
            Statement::AsyncBlock(async_block) => async_block.validate(_context),
            Statement::Block(block) => block.validate(_context),
            Statement::Break => Ok(()),
            Statement::Expression(expression) => expression.validate(_context),
            Statement::IfElse(if_else) => if_else.validate(_context),
            Statement::Loop(r#loop) => r#loop.validate(_context),
            Statement::While(r#while) => r#while.validate(_context),
            Statement::StructureDefinition(structure_definition) => {
                structure_definition.validate(_context)
            }
        }
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        match self {
            Statement::Assignment(assignment) => assignment.run(_context),
            Statement::AsyncBlock(async_block) => async_block.run(_context),
            Statement::Block(block) => block.run(_context),
            Statement::Break => Ok(Action::Break),
            Statement::Expression(expression) => expression.run(_context),
            Statement::IfElse(if_else) => if_else.run(_context),
            Statement::Loop(r#loop) => r#loop.run(_context),
            Statement::While(r#while) => r#while.run(_context),
            Statement::StructureDefinition(structure_definition) => {
                structure_definition.run(_context)
            }
        }
    }
}
