use std::{
    collections::BTreeMap,
    fmt::{self, Debug, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{value_node::ValueNode, Expression, Value};

/// The type of a `Value`.
#[derive(Clone, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ValueType {
    Any,
    String,
    Float,
    Integer,
    Boolean,
    ListExact(Vec<Expression>),
    Empty,
    Map(BTreeMap<String, ValueNode>),
    Table,
    Function,
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
            (ValueType::Empty, ValueType::Empty) => true,
            (ValueType::Map(left), ValueType::Map(right)) => left == right,
            (ValueType::Table, ValueType::Table) => true,
            (ValueType::Function, ValueType::Function) => true,
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
            ValueType::ListExact(list) => {
                write!(f, "(")?;
                for (index, item) in list.into_iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{item:?}")?;
                }

                write!(f, ")")
            }
            ValueType::Empty => write!(f, "empty"),
            ValueType::Map(_map) => write!(f, "map"),
            ValueType::Table => write!(f, "table"),
            ValueType::Function => write!(f, "function"),
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
                let value_nodes = list
                    .iter()
                    .map(|value| Expression::Value(ValueNode::new(value.value_type(), 0, 0)))
                    .collect();

                ValueType::ListExact(value_nodes)
            }
            Value::Map(map) => {
                let mut value_nodes = BTreeMap::new();

                for (key, value) in map.inner() {
                    let value_type = ValueType::from(value);
                    let value_node = ValueNode::new(value_type, 0, 0);

                    value_nodes.insert(key.to_string(), value_node);
                }

                ValueType::Map(value_nodes)
            }
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
