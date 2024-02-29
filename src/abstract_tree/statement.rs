use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Assignment, Block, Expression, Loop, Type, Value};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Statement<'src> {
    Assignment(Assignment<'src>),
    Block(Block<'src>),
    Expression(Expression<'src>),
    Loop(Loop<'src>),
}

impl<'src> AbstractTree for Statement<'src> {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        todo!()
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        todo!()
    }

    fn run(self, _context: &Context) -> Result<Value, RuntimeError> {
        match self {
            Statement::Assignment(assignment) => assignment.run(_context),
            Statement::Block(_) => todo!(),
            Statement::Expression(_) => todo!(),
            Statement::Loop(_) => todo!(),
        }
    }
}
