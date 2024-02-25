use crate::{context::Context, error::RuntimeError};

use super::{AbstractTree, Block, Value};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Loop {
    block: Block,
}

impl AbstractTree for Loop {
    fn run(self, _: &Context) -> Result<Value, RuntimeError> {
        todo!()
    }
}
