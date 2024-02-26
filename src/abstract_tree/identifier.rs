use crate::{context::Context, error::RuntimeError, Value};

use super::AbstractTree;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Identifier<'src>(&'src str);

impl<'src> Identifier<'src> {
    pub fn new(string: &'src str) -> Self {
        Identifier(string)
    }
}

impl<'src> AbstractTree for Identifier<'src> {
    fn run(self, _context: &Context) -> Result<Value, RuntimeError> {
        todo!()
        // let value = context.get(&self)?.unwrap_or_else(Value::none).clone();

        // Ok(value)
    }
}
