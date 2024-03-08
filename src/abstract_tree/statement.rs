use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Assignment, Block, Expression, IfElse, Loop, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Statement<'src> {
    Assignment(Assignment<'src>),
    Block(Block<'src>),
    Break(Expression<'src>),
    Expression(Expression<'src>),
    IfElse(IfElse<'src>),
    Loop(Loop<'src>),
}

impl<'src> AbstractTree for Statement<'src> {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match self {
            Statement::Assignment(assignment) => assignment.expected_type(_context),
            Statement::Block(block) => block.expected_type(_context),
            Statement::Break(expression) => expression.expected_type(_context),
            Statement::Expression(expression) => expression.expected_type(_context),
            Statement::IfElse(if_else) => if_else.expected_type(_context),
            Statement::Loop(r#loop) => r#loop.expected_type(_context),
        }
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        match self {
            Statement::Assignment(assignment) => assignment.validate(_context),
            Statement::Block(block) => block.validate(_context),
            Statement::Break(expression) => expression.validate(_context),
            Statement::Expression(expression) => expression.validate(_context),
            Statement::IfElse(if_else) => if_else.validate(_context),
            Statement::Loop(r#loop) => r#loop.validate(_context),
        }
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        match self {
            Statement::Assignment(assignment) => assignment.run(_context),
            Statement::Block(block) => block.run(_context),
            Statement::Break(expression) => {
                let value = expression.run(_context)?.as_return_value()?;

                Ok(Action::Break(value))
            }
            Statement::Expression(expression) => expression.run(_context),
            Statement::IfElse(if_else) => if_else.run(_context),
            Statement::Loop(r#loop) => r#loop.run(_context),
        }
    }
}
