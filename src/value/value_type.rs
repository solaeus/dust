use std::fmt::Display;

use crate::Value;

/// The type of a `Value`.
#[derive(Clone, Debug)]
pub enum ValueType {
    Any,
    String,
    Float,
    Int,
    Boolean,
    List(Vec<ValueType>),
    Empty,
    Map,
    Table,
    Function,
    Time,
}

impl Eq for ValueType {}

impl PartialEq for ValueType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ValueType::Any, _) => true,
            (_, ValueType::Any) => true,
            (ValueType::String, ValueType::String) => true,
            (ValueType::Float, ValueType::Float) => true,
            (ValueType::Int, ValueType::Int) => true,
            (ValueType::Boolean, ValueType::Boolean) => true,
            (ValueType::List(left), ValueType::List(right)) => left == right,
            (ValueType::Empty, ValueType::Empty) => true,
            (ValueType::Map, ValueType::Map) => true,
            (ValueType::Table, ValueType::Table) => true,
            (ValueType::Function, ValueType::Function) => true,
            (ValueType::Time, ValueType::Time) => true,
            _ => false,
        }
    }
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match &self {
            ValueType::Any => "any",
            ValueType::String => "string",
            ValueType::Float => "float",
            ValueType::Int => "integer",
            ValueType::Boolean => "boolean",
            ValueType::List(_) => "list",
            ValueType::Empty => "empty",
            ValueType::Map => "map",
            ValueType::Table => "table",
            ValueType::Function => "function",
            ValueType::Time => "time",
        };

        write!(f, "{text}")
    }
}

impl From<&Value> for ValueType {
    fn from(value: &Value) -> Self {
        match value {
            Value::String(_) => ValueType::String,
            Value::Float(_) => ValueType::Float,
            Value::Integer(_) => ValueType::Int,
            Value::Boolean(_) => ValueType::Boolean,
            Value::List(list) => {
                ValueType::List(list.iter().map(|value| value.value_type()).collect())
            }
            Value::Empty => ValueType::Empty,
            Value::Map(_) => ValueType::Map,
            Value::Table { .. } => ValueType::Table,
            Value::Function(_) => ValueType::Function,
            Value::Time(_) => ValueType::Time,
        }
    }
}

impl From<&mut Value> for ValueType {
    fn from(value: &mut Value) -> Self {
        From::<&Value>::from(value)
    }
}
