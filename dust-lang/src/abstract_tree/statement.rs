use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{
    AbstractNode, Action, Assignment, AsyncBlock, Block, Expression, IfElse, Loop, SourcePosition,
    StructureDefinition, Type, While, WithPosition,
};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Statement {
    Assignment(WithPosition<Assignment>),
    AsyncBlock(WithPosition<AsyncBlock>),
    Block(WithPosition<Block>),
    Break(WithPosition<()>),
    Expression(Expression),
    IfElse(WithPosition<IfElse>),
    Loop(WithPosition<Loop>),
    StructureDefinition(WithPosition<StructureDefinition>),
    While(WithPosition<While>),
}

impl Statement {
    pub fn position(&self) -> SourcePosition {
        match self {
            Statement::Assignment(inner) => inner.position,
            Statement::AsyncBlock(inner) => inner.position,
            Statement::Block(inner) => inner.position,
            Statement::Break(inner) => inner.position,
            Statement::Expression(expression) => expression.position(),
            Statement::IfElse(inner) => inner.position,
            Statement::Loop(inner) => inner.position,
            Statement::StructureDefinition(inner) => inner.position,
            Statement::While(inner) => inner.position,
        }
    }
}

impl AbstractNode for Statement {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match self {
            Statement::Assignment(assignment) => assignment.node.expected_type(_context),
            Statement::AsyncBlock(async_block) => async_block.node.expected_type(_context),
            Statement::Block(block) => block.node.expected_type(_context),
            Statement::Break(_) => Ok(Type::None),
            Statement::Expression(expression) => expression.expected_type(_context),
            Statement::IfElse(if_else) => if_else.node.expected_type(_context),
            Statement::Loop(r#loop) => r#loop.node.expected_type(_context),
            Statement::While(r#while) => r#while.node.expected_type(_context),
            Statement::StructureDefinition(structure_definition) => {
                structure_definition.node.expected_type(_context)
            }
        }
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        match self {
            Statement::Assignment(assignment) => assignment.node.validate(_context),
            Statement::AsyncBlock(async_block) => async_block.node.validate(_context),
            Statement::Block(block) => block.node.validate(_context),
            Statement::Break(_) => Ok(()),
            Statement::Expression(expression) => expression.validate(_context),
            Statement::IfElse(if_else) => if_else.node.validate(_context),
            Statement::Loop(r#loop) => r#loop.node.validate(_context),
            Statement::While(r#while) => r#while.node.validate(_context),
            Statement::StructureDefinition(structure_definition) => {
                structure_definition.node.validate(_context)
            }
        }
    }

    fn run(self, context: &mut Context, clear_variables: bool) -> Result<Action, RuntimeError> {
        let result = match self {
            Statement::Assignment(assignment) => assignment.node.run(context, clear_variables),
            Statement::AsyncBlock(async_block) => async_block.node.run(context, clear_variables),
            Statement::Block(block) => block.node.run(context, clear_variables),
            Statement::Break(_) => Ok(Action::Break),
            Statement::Expression(expression) => expression.run(context, clear_variables),
            Statement::IfElse(if_else) => if_else.node.run(context, clear_variables),
            Statement::Loop(r#loop) => r#loop.node.run(context, clear_variables),
            Statement::While(r#while) => r#while.node.run(context, clear_variables),
            Statement::StructureDefinition(structure_definition) => {
                structure_definition.node.run(context, clear_variables)
            }
        };

        if clear_variables {
            context.clean()?;
        }

        result
    }
}
