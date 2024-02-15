use crate::{Identifier, Value};

#[derive(Debug, Clone)]
pub struct Enum {
    variant_name: String,
    value: Box<Value>,
}

impl Enum {
    pub fn new(variant_name: String, value: Value) -> Self {
        Self {
            variant_name,
            value: Box::new(value),
        }
    }
}
