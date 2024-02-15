use std::fmt::{self, Display, Formatter};

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

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn variant_name(&self) -> &String {
        &self.variant_name
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

impl Display for EnumInstance {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}::{}({})", self.name, self.variant_name, self.value)
    }
}
