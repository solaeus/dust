use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Identifier, Value};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EnumInstance {
    name: Identifier,
    variant: Identifier,
    value: Option<Box<Value>>,
}

impl EnumInstance {
    pub fn new(name: Identifier, variant_name: Identifier, value: Option<Value>) -> Self {
        Self {
            name,
            variant: variant_name,
            value: value.map(|value| Box::new(value)),
        }
    }

    pub fn name(&self) -> &Identifier {
        &self.name
    }

    pub fn variant(&self) -> &Identifier {
        &self.variant
    }

    pub fn value(&self) -> &Option<Box<Value>> {
        &self.value
    }
}

impl Display for EnumInstance {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}::{}({:?})", self.name, self.variant, self.value)
    }
}
