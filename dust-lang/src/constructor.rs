use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{Identifier, Struct, Value};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Constructor {
    Unit(UnitConstructor),
    Tuple(TupleConstructor),
    Fields(FieldsConstructor),
}

impl Display for Constructor {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Constructor::Unit(unit) => write!(f, "{}", unit.name),
            Constructor::Tuple(tuple) => write!(f, "{}", tuple.name),
            Constructor::Fields(fields) => write!(f, "{}", fields.name),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UnitConstructor {
    pub name: Identifier,
}

impl UnitConstructor {
    pub fn construct(self) -> Value {
        Value::r#struct(Struct::Unit { name: self.name })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TupleConstructor {
    pub name: Identifier,
}

impl TupleConstructor {
    pub fn construct(self, fields: Vec<Value>) -> Value {
        Value::r#struct(Struct::Tuple {
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
        Value::r#struct(Struct::Fields {
            name: self.name,
            fields,
        })
    }
}
