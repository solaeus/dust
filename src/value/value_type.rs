use std::fmt::{self, Debug, Display, Formatter};

use crate::Value;

/// The type of a `Value`.
#[derive(Clone)]
pub enum ValueType {
    Any,
    String,
    Float,
    Integer,
    Boolean,
    List,
    ListOf(Box<ValueType>),
    ListExact(Vec<ValueType>),
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
            (ValueType::ListExact(left), ValueType::ListExact(right)) => left == right,
            (ValueType::ListExact(_), ValueType::List) => true,
            (ValueType::List, ValueType::ListExact(_)) => true,
            (ValueType::ListOf(left), ValueType::ListOf(right)) => left == right,
            (ValueType::ListOf(_), ValueType::List) => true,
            (ValueType::List, ValueType::ListOf(_)) => true,
            (ValueType::ListOf(value_type), ValueType::ListExact(exact_list))
            | (ValueType::ListExact(exact_list), ValueType::ListOf(value_type)) => exact_list
                .iter()
                .all(|exact_type| exact_type == value_type.as_ref()),
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            ValueType::Any => write!(f, "any"),
            ValueType::String => write!(f, "string"),
            ValueType::Float => write!(f, "float"),
            ValueType::Integer => write!(f, "integer"),
            ValueType::Boolean => write!(f, "boolean"),
            ValueType::List => write!(f, "list"),
            ValueType::ListOf(value_type) => {
                write!(f, "({value_type}s)")
            }
            ValueType::ListExact(list) => {
                write!(f, "(")?;
                for (index, item) in list.into_iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{item}")?;
                }

                write!(f, ")")
            }
            ValueType::Empty => write!(f, "empty"),
            ValueType::Map => write!(f, "map"),
            ValueType::Table => write!(f, "table"),
            ValueType::Function => write!(f, "function"),
            ValueType::Time => write!(f, "time"),
        }
    }
}

impl Debug for ValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl From<&Value> for ValueType {
    fn from(value: &Value) -> Self {
        match value {
            Value::String(_) => ValueType::String,
            Value::Float(_) => ValueType::Float,
            Value::Integer(_) => ValueType::Integer,
            Value::Boolean(_) => ValueType::Boolean,
            Value::Empty => ValueType::Empty,
            Value::List(list) => {
                let values = list.iter().map(|value| value.value_type()).collect();

                ValueType::ListExact(values)
            }
            Value::Map(_) => ValueType::Map,
            Value::Table { .. } => ValueType::Table,
            Value::Function(_) => ValueType::Function,
        }
    }
}

impl From<&mut Value> for ValueType {
    fn from(value: &mut Value) -> Self {
        From::<&Value>::from(value)
    }
}
