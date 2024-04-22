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
            Statement::Assignment(assignment) => assignment.item.expected_type(_context),
            Statement::AsyncBlock(async_block) => async_block.item.expected_type(_context),
            Statement::Block(block) => block.item.expected_type(_context),
            Statement::Break(_) => Ok(Type::None),
            Statement::Expression(expression) => expression.expected_type(_context),
            Statement::IfElse(if_else) => if_else.item.expected_type(_context),
            Statement::Loop(r#loop) => r#loop.item.expected_type(_context),
            Statement::While(r#while) => r#while.item.expected_type(_context),
            Statement::StructureDefinition(structure_definition) => {
                structure_definition.item.expected_type(_context)
            }
        }
    }

    fn validate(&self, _context: &Context, _manage_memory: bool) -> Result<(), ValidationError> {
        match self {
            Statement::Assignment(assignment) => assignment.item.validate(_context, _manage_memory),
            Statement::AsyncBlock(async_block) => {
                async_block.item.validate(_context, _manage_memory)
            }
            Statement::Block(block) => block.item.validate(_context, _manage_memory),
            Statement::Break(_) => Ok(()),
            Statement::Expression(expression) => expression.validate(_context, _manage_memory),
            Statement::IfElse(if_else) => if_else.item.validate(_context, _manage_memory),
            Statement::Loop(r#loop) => r#loop.item.validate(_context, _manage_memory),
            Statement::While(r#while) => r#while.item.validate(_context, _manage_memory),
            Statement::StructureDefinition(structure_definition) => {
                structure_definition.item.validate(_context, _manage_memory)
            }
        }
    }

    fn run(self, context: &mut Context, manage_memory: bool) -> Result<Action, RuntimeError> {
        let result = match self {
            Statement::Assignment(assignment) => assignment.item.run(context, manage_memory),
            Statement::AsyncBlock(async_block) => async_block.item.run(context, manage_memory),
            Statement::Block(block) => block.item.run(context, manage_memory),
            Statement::Break(_) => Ok(Action::Break),
            Statement::Expression(expression) => expression.run(context, manage_memory),
            Statement::IfElse(if_else) => if_else.item.run(context, manage_memory),
            Statement::Loop(r#loop) => r#loop.item.run(context, manage_memory),
            Statement::While(r#while) => r#while.item.run(context, manage_memory),
            Statement::StructureDefinition(structure_definition) => {
                structure_definition.item.run(context, manage_memory)
            }
        };

        if manage_memory {
            context.clean()?;
        }

        result
    }
}
