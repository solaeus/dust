use crate::{context::Context, error::RuntimeError};

use super::{AbstractTree, Assignment, Block, Identifier, Logic, Loop, Value};

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Assignment(Assignment),
    Block(Block),
    Identifier(Identifier),
    Loop(Loop),
    Value(Value),
    Logic(Box<Logic>),
}

impl AbstractTree for Statement {
    fn run(self, _context: &Context) -> Result<Value, RuntimeError> {
        match self {
            Statement::Assignment(assignment) => assignment.run(_context),
            Statement::Block(_) => todo!(),
            Statement::Identifier(identifier) => identifier.run(_context),
            Statement::Loop(_) => todo!(),
            Statement::Value(value) => value.run(_context),
            Statement::Logic(_) => todo!(),
        }
    }
}
