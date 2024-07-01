use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{
    AbstractNode, Assignment, AsyncBlock, Block, EnumDeclaration, Evaluation, Expression, IfElse,
    Loop, SourcePosition, StructureDefinition, Type, TypeAlias, Use, While, WithPosition,
};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Statement {
    Assignment(WithPosition<Assignment>),
    AsyncBlock(WithPosition<AsyncBlock>),
    Block(WithPosition<Block>),
    Break(WithPosition<()>),
    IfElse(WithPosition<IfElse>),
    Loop(WithPosition<Loop>),
    Null(WithPosition<()>),
    StructureDefinition(WithPosition<StructureDefinition>),
    TypeAlias(WithPosition<TypeAlias>),
    EnumDeclaration(WithPosition<EnumDeclaration>),
    Expression(Expression),
    While(WithPosition<While>),
    Use(WithPosition<Use>),
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
            Statement::Null(inner) => inner.position,
            Statement::StructureDefinition(inner) => inner.position,
            Statement::TypeAlias(inner) => inner.position,
            Statement::EnumDeclaration(inner) => inner.position,
            Statement::While(inner) => inner.position,
            Statement::Use(inner) => inner.position,
        }
    }

    pub fn last_evaluated_statement(&self) -> &Self {
        match self {
            Statement::Block(inner) => inner.node.last_statement(),
            Statement::Loop(inner) => inner.node.last_statement(),
            statement => statement,
        }
    }
}

impl AbstractNode for Statement {
    fn define_types(&self, _context: &Context) -> Result<(), ValidationError> {
        log::trace!("Defining types for statement at {}", self.position());

        match self {
            Statement::Expression(expression) => expression.define_types(_context),
            Statement::IfElse(if_else) => if_else.node.define_types(_context),
            Statement::Block(block) => block.node.define_types(_context),
            Statement::AsyncBlock(async_block) => async_block.node.define_types(_context),
            Statement::Assignment(assignment) => assignment.node.define_types(_context),
            Statement::Loop(r#loop) => r#loop.node.define_types(_context),
            Statement::StructureDefinition(struct_definition) => {
                struct_definition.node.define_types(_context)
            }
            Statement::TypeAlias(type_alias) => type_alias.node.define_types(_context),
            Statement::EnumDeclaration(enum_declaration) => {
                enum_declaration.node.define_types(_context)
            }
            Statement::While(r#while) => r#while.node.define_types(_context),
            Statement::Use(r#use) => r#use.node.define_types(_context),
            Statement::Break(_) | Statement::Null(_) => Ok(()),
        }
    }

    fn validate(&self, _context: &Context, _manage_memory: bool) -> Result<(), ValidationError> {
        log::trace!("Validating statement at {}", self.position());

        match self {
            Statement::Assignment(assignment) => assignment.node.validate(_context, _manage_memory),
            Statement::AsyncBlock(async_block) => {
                async_block.node.validate(_context, _manage_memory)
            }
            Statement::Block(block) => block.node.validate(_context, _manage_memory),
            Statement::Expression(expression) => expression.validate(_context, _manage_memory),
            Statement::IfElse(if_else) => if_else.node.validate(_context, _manage_memory),
            Statement::Loop(r#loop) => r#loop.node.validate(_context, _manage_memory),
            Statement::While(r#while) => r#while.node.validate(_context, _manage_memory),
            Statement::Use(r#use) => r#use.node.validate(_context, _manage_memory),
            _ => Ok(()),
        }
    }

    fn evaluate(
        self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        log::trace!("Evaluating statement at {}", self.position());

        let result = match self {
            Statement::Assignment(assignment) => assignment.node.evaluate(context, manage_memory),
            Statement::AsyncBlock(async_block) => async_block.node.evaluate(context, manage_memory),
            Statement::Block(block) => block.node.evaluate(context, manage_memory),
            Statement::Break(_) => Ok(Some(Evaluation::Break)),
            Statement::Expression(expression) => expression.evaluate(context, manage_memory),
            Statement::IfElse(if_else) => if_else.node.evaluate(context, manage_memory),
            Statement::Loop(r#loop) => r#loop.node.evaluate(context, manage_memory),
            Statement::Null(_) => Ok(None),
            Statement::StructureDefinition(structure_definition) => {
                structure_definition.node.evaluate(context, manage_memory)
            }
            Statement::TypeAlias(type_alias) => type_alias.node.evaluate(context, manage_memory),
            Statement::EnumDeclaration(type_alias) => {
                type_alias.node.evaluate(context, manage_memory)
            }
            Statement::While(r#while) => r#while.node.evaluate(context, manage_memory),
            Statement::Use(r#use) => r#use.node.evaluate(context, manage_memory),
        };

        if manage_memory {
            context.clean()?;
        }

        result
    }

    fn expected_type(&self, _context: &Context) -> Result<Option<Type>, ValidationError> {
        match self {
            Statement::Expression(expression) => expression.expected_type(_context),
            Statement::IfElse(if_else) => if_else.node.expected_type(_context),
            Statement::Block(block) => block.node.expected_type(_context),
            Statement::AsyncBlock(async_block) => async_block.node.expected_type(_context),
            Statement::Assignment(assignment) => assignment.node.expected_type(_context),
            Statement::Loop(r#loop) => r#loop.node.expected_type(_context),
            Statement::StructureDefinition(struct_definition) => {
                struct_definition.node.expected_type(_context)
            }
            Statement::TypeAlias(type_alias) => type_alias.node.expected_type(_context),
            Statement::EnumDeclaration(enum_declaration) => {
                enum_declaration.node.expected_type(_context)
            }
            Statement::While(r#while) => r#while.node.expected_type(_context),
            Statement::Use(r#use) => r#use.node.expected_type(_context),
            Statement::Break(_) | Statement::Null(_) => Ok(None),
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Statement::Assignment(inner) => write!(f, "{}", inner.node),
            Statement::AsyncBlock(inner) => write!(f, "{}", inner.node),
            Statement::Block(inner) => write!(f, "{}", inner.node),
            Statement::Break(_) => write!(f, "break"),
            Statement::IfElse(inner) => write!(f, "{}", inner.node),
            Statement::Loop(inner) => write!(f, "{}", inner.node),
            Statement::Null(_) => write!(f, ";"),
            Statement::StructureDefinition(inner) => write!(f, "{}", inner.node),
            Statement::TypeAlias(inner) => write!(f, "{}", inner.node),
            Statement::EnumDeclaration(inner) => write!(f, "{}", inner.node),
            Statement::Expression(expression) => write!(f, "{expression}"),
            Statement::While(inner) => write!(f, "{}", inner.node),
            Statement::Use(inner) => write!(f, "{}", inner.node),
        }
    }
}
