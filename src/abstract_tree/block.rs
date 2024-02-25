use crate::{context::Context, error::RuntimeError};

use super::{AbstractTree, Statement, Value};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Block {
    statements: Vec<Statement>,
}

impl Block {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}

impl AbstractTree for Block {
    fn run(self, _: &Context) -> Result<Value, RuntimeError> {
        todo!()
    }
}
