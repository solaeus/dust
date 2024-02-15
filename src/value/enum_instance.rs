use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::Value;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EnumInstance {
    name: String,
    variant_name: String,
    value: Option<Box<Value>>,
}

impl EnumInstance {
    pub fn new(name: String, variant_name: String, value: Option<Value>) -> Self {
        Self {
            name,
            variant_name,
            value: value.map(|value| Box::new(value)),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn variant_name(&self) -> &String {
        &self.variant_name
    }

    pub fn value(&self) -> &Option<Box<Value>> {
        &self.value
    }
}

impl Display for EnumInstance {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}::{}({:?})", self.name, self.variant_name, self.value)
    }
}
