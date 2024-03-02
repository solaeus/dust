use std::sync::Arc;

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    Value,
};

use super::{AbstractTree, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Identifier(Arc<String>);

impl Identifier {
    pub fn new<T: ToString>(string: T) -> Self {
        Identifier(Arc::new(string.to_string()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl AbstractTree for Identifier {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        todo!()
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        todo!()
    }

    fn run(self, _context: &Context) -> Result<Value, RuntimeError> {
        todo!()
        // let value = context.get(&self)?.unwrap_or_else(Value::none).clone();

        // Ok(value)
    }
}
