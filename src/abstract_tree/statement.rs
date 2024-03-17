use chumsky::span::{SimpleSpan, Span};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Assignment, Block, Expression, IfElse, Loop, Type, While};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Statement {
    Assignment(Assignment),
    Block(Block),
    Break,
    Expression(Expression),
    IfElse(IfElse),
    Loop(Loop),
    While(While),
}

impl AbstractTree for Statement {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match self {
            StatementInner::Assignment(assignment) => assignment.expected_type(_context),
            StatementInner::Block(block) => block.expected_type(_context),
            StatementInner::Break => Ok(Type::None),
            StatementInner::Expression(expression) => expression.expected_type(_context),
            StatementInner::IfElse(if_else) => if_else.expected_type(_context),
            StatementInner::Loop(r#loop) => r#loop.expected_type(_context),
            StatementInner::While(r#while) => r#while.expected_type(_context),
        }
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        match self {
            StatementInner::Assignment(assignment) => assignment.validate(_context),
            StatementInner::Block(block) => block.validate(_context),
            StatementInner::Break => Ok(()),
            StatementInner::Expression(expression) => expression.validate(_context),
            StatementInner::IfElse(if_else) => if_else.validate(_context),
            StatementInner::Loop(r#loop) => r#loop.validate(_context),
            StatementInner::While(r#while) => r#while.validate(_context),
        }
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        match self {
            StatementInner::Assignment(assignment) => assignment.run(_context),
            StatementInner::Block(block) => block.run(_context),
            StatementInner::Break => Ok(Action::Break),
            StatementInner::Expression(expression) => expression.run(_context),
            StatementInner::IfElse(if_else) => if_else.run(_context),
            StatementInner::Loop(r#loop) => r#loop.run(_context),
            StatementInner::While(r#while) => r#while.run(_context),
        }
    }
}
