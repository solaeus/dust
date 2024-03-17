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
            Statement::Assignment(assignment) => assignment.expected_type(_context),
            Statement::Block(block) => block.expected_type(_context),
            Statement::Break => Ok(Type::None),
            Statement::Expression(expression) => expression.expected_type(_context),
            Statement::IfElse(if_else) => if_else.expected_type(_context),
            Statement::Loop(r#loop) => r#loop.expected_type(_context),
            Statement::While(r#while) => r#while.expected_type(_context),
        }
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        match self {
            Statement::Assignment(assignment) => assignment.validate(_context),
            Statement::Block(block) => block.validate(_context),
            Statement::Break => Ok(()),
            Statement::Expression(expression) => expression.validate(_context),
            Statement::IfElse(if_else) => if_else.validate(_context),
            Statement::Loop(r#loop) => r#loop.validate(_context),
            Statement::While(r#while) => r#while.validate(_context),
        }
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        match self {
            Statement::Assignment(assignment) => assignment.run(_context),
            Statement::Block(block) => block.run(_context),
            Statement::Break => Ok(Action::Break),
            Statement::Expression(expression) => expression.run(_context),
            Statement::IfElse(if_else) => if_else.run(_context),
            Statement::Loop(r#loop) => r#loop.run(_context),
            Statement::While(r#while) => r#while.run(_context),
        }
    }
}
