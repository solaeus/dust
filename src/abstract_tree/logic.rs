use crate::{context::Context, error::RuntimeError};

use super::{AbstractTree, Statement, Value};

#[derive(Clone, Debug, PartialEq)]
pub enum Logic {
    Equal(Statement, Statement),
    NotEqual(Statement, Statement),
    Greater(Statement, Statement),
    Less(Statement, Statement),
    GreaterOrEqual(Statement, Statement),
    LessOrEqual(Statement, Statement),
    And(Statement, Statement),
    Or(Statement, Statement),
    Not(Statement),
}

impl AbstractTree for Logic {
    fn run(self, _: &Context) -> Result<Value, RuntimeError> {
        todo!()
    }
}
