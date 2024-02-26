use crate::{context::Context, error::RuntimeError};

use super::{AbstractTree, Assignment, Block, Expression, Loop, Value};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Statement<'src> {
    Assignment(Assignment<'src>),
    Block(Block<'src>),
    Expression(Expression<'src>),
    Loop(Loop<'src>),
}

impl<'src> AbstractTree for Statement<'src> {
    fn run(self, _context: &Context) -> Result<Value, RuntimeError> {
        match self {
            Statement::Assignment(assignment) => assignment.run(_context),
            Statement::Block(_) => todo!(),
            Statement::Expression(_) => todo!(),
            Statement::Loop(_) => todo!(),
        }
    }
}
