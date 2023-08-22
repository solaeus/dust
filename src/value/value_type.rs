use std::fmt::Display;

use crate::Value;

/// The type of a `Value`.
#[derive(Clone, Debug)]
pub enum ValueType {
    Any,
    String,
    Float,
    Integer,
    Boolean,
    List,
    ListOf(Vec<ValueType>),
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
            (ValueType::Integer, ValueType::Integer) => true,
            (ValueType::Boolean, ValueType::Boolean) => true,
            (ValueType::ListOf(left), ValueType::ListOf(right)) => left == right,
            (ValueType::ListOf(_), ValueType::List) => true,
            (ValueType::List, ValueType::ListOf(_)) => true,
            (ValueType::List, ValueType::List) => true,
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
        match &self {
            ValueType::Any => write!(f, "any"),
            ValueType::String => write!(f, "string"),
            ValueType::Float => write!(f, "float"),
            ValueType::Integer => write!(f, "integer"),
            ValueType::Boolean => write!(f, "boolean"),
            ValueType::List => write!(f, "list"),
            ValueType::ListOf(items) => {
                let items = items
                    .iter()
                    .map(|value_type| value_type.to_string() + " ")
                    .collect::<String>();

                write!(f, "list of {items}")
            }
            ValueType::Empty => write!(f, "empty"),
            ValueType::Map => write!(f, "map"),
            ValueType::Table => write!(f, "table"),
            ValueType::Function => write!(f, "function"),
            ValueType::Time => write!(f, "time"),
        }
    }
}

impl From<&Value> for ValueType {
    fn from(value: &Value) -> Self {
        match value {
            Value::String(_) => ValueType::String,
            Value::Float(_) => ValueType::Float,
            Value::Integer(_) => ValueType::Integer,
            Value::Boolean(_) => ValueType::Boolean,
            Value::List(list) => {
                ValueType::ListOf(list.iter().map(|value| value.value_type()).collect())
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
