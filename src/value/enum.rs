use crate::Value;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct EnumInstance {
    name: String,
    variant_name: String,
    value: Box<Value>,
}

impl EnumInstance {
    pub fn new(name: String, variant_name: String, value: Value) -> Self {
        Self {
            name,
            variant_name,
            value: Box::new(value),
        }
    }
}
