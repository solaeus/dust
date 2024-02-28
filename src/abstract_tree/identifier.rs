use std::sync::Arc;

use crate::{context::Context, error::RuntimeError, Value};

use super::AbstractTree;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Identifier(Arc<String>);

impl Identifier {
    pub fn new<T: ToString>(string: T) -> Self {
        Identifier(Arc::new(string.to_string()))
    }
}

impl AbstractTree for Identifier {
    fn run(self, _context: &Context) -> Result<Value, RuntimeError> {
        todo!()
        // let value = context.get(&self)?.unwrap_or_else(Value::none).clone();

        // Ok(value)
    }
}
