use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Identifier, Struct, Value};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Constructor {
    Unit(UnitConstructor),
    Tuple(TupleConstructor),
    Fields(FieldsConstructor),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UnitConstructor {
    pub name: Identifier,
}

impl UnitConstructor {
    pub fn construct(self) -> Value {
        Value::Struct(Struct::Unit { name: self.name })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TupleConstructor {
    pub name: Identifier,
}

impl TupleConstructor {
    pub fn construct(self, fields: Vec<Value>) -> Value {
        Value::Struct(Struct::Tuple {
            name: self.name,
            fields,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FieldsConstructor {
    pub name: Identifier,
}

impl FieldsConstructor {
    pub fn construct(self, fields: HashMap<Identifier, Value>) -> Value {
        Value::Struct(Struct::Fields {
            name: self.name,
            fields,
        })
    }
}
