use std::{
    collections::BTreeMap,
    ops::Range,
    sync::{Arc, OnceLock},
};

use crate::{context::Context, error::RuntimeError};

use super::{AbstractTree, Identifier, Statement};

pub static NONE: OnceLock<Value> = OnceLock::new();

#[derive(Clone, Debug, PartialEq)]
pub struct Value(Arc<ValueInner>);

impl Value {
    pub fn inner(&self) -> &Arc<ValueInner> {
        &self.0
    }

    pub fn none() -> Self {
        NONE.get_or_init(|| {
            Value::r#enum(EnumInstance {
                type_name: Identifier::new("Option"),
                variant: Identifier::new("None"),
            })
        })
        .clone()
    }

    pub fn boolean(boolean: bool) -> Self {
        Value(Arc::new(ValueInner::Boolean(boolean)))
    }

    pub fn float(float: f64) -> Self {
        Value(Arc::new(ValueInner::Float(float)))
    }

    pub fn integer(integer: i64) -> Self {
        Value(Arc::new(ValueInner::Integer(integer)))
    }

    pub fn list(list: Vec<Statement>) -> Self {
        Value(Arc::new(ValueInner::List(list)))
    }

    pub fn map(map: BTreeMap<Identifier, Value>) -> Self {
        Value(Arc::new(ValueInner::Map(map)))
    }

    pub fn range(range: Range<i64>) -> Self {
        Value(Arc::new(ValueInner::Range(range)))
    }

    pub fn string<T: ToString>(string: T) -> Self {
        Value(Arc::new(ValueInner::String(string.to_string())))
    }

    pub fn r#enum(r#enum: EnumInstance) -> Self {
        Value(Arc::new(ValueInner::Enum(r#enum)))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueInner {
    Boolean(bool),
    Float(f64),
    Integer(i64),
    List(Vec<Statement>),
    Map(BTreeMap<Identifier, Value>),
    Range(Range<i64>),
    String(String),
    Enum(EnumInstance),
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnumInstance {
    type_name: Identifier,
    variant: Identifier,
}

impl AbstractTree for Value {
    fn run(self, _: &Context) -> Result<Value, RuntimeError> {
        Ok(self)
    }
}
