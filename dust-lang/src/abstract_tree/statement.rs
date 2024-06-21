use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{
    Assignment, AsyncBlock, Block, EnumDeclaration, Evaluate, Evaluation, ExpectedType, Expression,
    IfElse, Loop, Run, SourcePosition, StructureDefinition, Type, TypeAlias, Validate, While,
    WithPosition,
};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Statement {
    Assignment(WithPosition<Assignment>),
    AsyncBlock(WithPosition<AsyncBlock>),
    Block(WithPosition<Block>),
    Break(WithPosition<()>),
    IfElse(WithPosition<IfElse>),
    Loop(WithPosition<Loop>),
    StructureDefinition(WithPosition<StructureDefinition>),
    TypeAlias(WithPosition<TypeAlias>),
    EnumDeclaration(WithPosition<EnumDeclaration>),
    Expression(Expression),
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
            Statement::TypeAlias(inner) => inner.position,
            Statement::EnumDeclaration(inner) => inner.position,
            Statement::While(inner) => inner.position,
        }
    }

    pub fn last_child_statement(&self) -> &Self {
        match self {
            Statement::Block(inner) => inner.node.last_statement(),
            Statement::Loop(inner) => inner.node.last_statement(),
            statement => statement,
        }
    }
}

impl Run for Statement {
    fn run(
        self,
        context: &mut Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let result = match self {
            Statement::Assignment(assignment) => assignment.node.run(context, manage_memory),
            Statement::AsyncBlock(async_block) => async_block.node.run(context, manage_memory),
            Statement::Block(block) => block.node.run(context, manage_memory),
            Statement::Break(_) => Ok(Some(Evaluation::Break)),
            Statement::Expression(expression) => {
                let evaluation = expression.evaluate(context, manage_memory)?;

                Ok(Some(evaluation))
            }
            Statement::IfElse(if_else) => if_else.node.run(context, manage_memory),
            Statement::Loop(r#loop) => r#loop.node.run(context, manage_memory),
            Statement::StructureDefinition(structure_definition) => {
                structure_definition.node.run(context, manage_memory)
            }
            Statement::TypeAlias(type_alias) => type_alias.node.run(context, manage_memory),
            Statement::EnumDeclaration(type_alias) => type_alias.node.run(context, manage_memory),
            Statement::While(r#while) => r#while.node.run(context, manage_memory),
        };

        if manage_memory {
            context.clean()?;
        }

        result
    }
}

impl Validate for Statement {
    fn validate(
        &self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<(), ValidationError> {
        match self {
            Statement::Assignment(assignment) => assignment.node.validate(_context, _manage_memory),
            Statement::AsyncBlock(async_block) => {
                async_block.node.validate(_context, _manage_memory)
            }
            Statement::Block(block) => block.node.validate(_context, _manage_memory),
            Statement::Break(_) => Ok(()),
            Statement::Expression(expression) => expression.validate(_context, _manage_memory),
            Statement::IfElse(if_else) => if_else.node.validate(_context, _manage_memory),
            Statement::Loop(r#loop) => r#loop.node.validate(_context, _manage_memory),
            Statement::While(r#while) => r#while.node.validate(_context, _manage_memory),
            _ => Ok(()),
        }
    }
}

impl ExpectedType for Statement {
    fn expected_type(&self, _context: &mut Context) -> Result<Type, ValidationError> {
        match self {
            Statement::Expression(expression) => expression.expected_type(_context),
            Statement::IfElse(if_else) => if_else.node.expected_type(_context),
            Statement::Block(block) => block.node.expected_type(_context),
            Statement::AsyncBlock(async_block) => async_block.node.expected_type(_context),
            _ => Ok(Type::Void),
        }
    }
}
