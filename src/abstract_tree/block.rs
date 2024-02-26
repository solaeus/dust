use crate::{context::Context, error::RuntimeError, Value};

use super::{AbstractTree, Statement};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Block<'src> {
    statements: Vec<Statement<'src>>,
}

impl<'src> Block<'src> {
    pub fn new(statements: Vec<Statement<'src>>) -> Self {
        Self { statements }
    }
}

impl<'src> AbstractTree for Block<'src> {
    fn run(self, _: &Context) -> Result<Value, RuntimeError> {
        todo!()
    }
}
