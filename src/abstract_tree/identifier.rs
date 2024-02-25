use crate::{context::Context, error::RuntimeError};

use super::{AbstractTree, Value};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Identifier(String);

impl Identifier {
    pub fn new<T: ToString>(string: T) -> Identifier {
        Identifier(string.to_string())
    }
}

impl AbstractTree for Identifier {
    fn run(self, context: &Context) -> Result<Value, RuntimeError> {
        let value = context.get(&self)?.unwrap_or_else(Value::none).clone();

        Ok(value)
    }
}
