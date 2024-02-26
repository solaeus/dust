use crate::{context::Context, error::RuntimeError, Value};

use super::{AbstractTree, Block};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Loop<'src> {
    block: Block<'src>,
}

impl<'src> AbstractTree for Loop<'src> {
    fn run(self, _: &Context) -> Result<Value, RuntimeError> {
        todo!()
    }
}
