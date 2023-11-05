use std::{
    collections::BTreeMap,
    fmt::{self, Debug, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{value_node::ValueNode, Expression, Function, Identifier, Statement, Value};

/// The type of a `Value`.
#[derive(Clone, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ValueType {
    Any,
    String,
    Float,
    Integer,
    Boolean,
    List(Vec<Expression>),
    Empty,
    Map(BTreeMap<String, Statement>),
    Table {
        column_names: Vec<Identifier>,
        rows: Box<Expression>,
    },
    Function(Function),
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
            (ValueType::List(left), ValueType::List(right)) => left == right,
            (ValueType::Empty, ValueType::Empty) => true,
            (ValueType::Map(left), ValueType::Map(right)) => left == right,
            (
                ValueType::Table {
                    column_names: left_columns,
                    rows: left_rows,
                },
                ValueType::Table {
                    column_names: right_columns,
                    rows: right_rows,
                },
            ) => left_columns == right_columns && left_rows == right_rows,
            (ValueType::Function(left), ValueType::Function(right)) => left == right,
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
            ValueType::List(list) => {
                write!(f, "(")?;
                for (index, item) in list.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{item:?}")?;
                }

                write!(f, ")")
            }
            ValueType::Empty => write!(f, "empty"),
            ValueType::Map(_map) => write!(f, "map"),
            ValueType::Table {
                column_names: _,
                rows: _,
            } => {
                write!(f, "table")
            }
            ValueType::Function(function) => write!(f, "{function}"),
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
                    .items()
                    .iter()
                    .map(|value| Expression::Value(ValueNode::new(value.value_type(), 0, 0)))
                    .collect();

                ValueType::List(value_nodes)
            }
            Value::Map(map) => {
                let mut value_nodes = BTreeMap::new();

                for (key, value) in map.variables().unwrap().iter() {
                    let value_type = value.value_type();
                    let value_node = ValueNode::new(value_type, 0, 0);
                    let statement = Statement::Expression(Expression::Value(value_node));

                    value_nodes.insert(key.to_string(), statement);
                }

                ValueType::Map(value_nodes)
            }
            Value::Table(table) => ValueType::Table {
                column_names: table
                    .headers()
                    .iter()
                    .map(|column_name| Identifier::new(column_name.clone()))
                    .collect(),
                rows: Box::new(Expression::Value(ValueNode::new(
                    ValueType::List(Vec::with_capacity(0)),
                    0,
                    0,
                ))),
            },
            Value::Function(function) => ValueType::Function(function.clone()),
        }
    }
}

impl From<&mut Value> for ValueType {
    fn from(value: &mut Value) -> Self {
        From::<&Value>::from(value)
    }
}
