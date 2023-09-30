//! Types that represent runtime values.
use crate::{
    error::{Error, Result},
    Expression, Function, Statement, Table, Time, ValueType, VariableMap,
};

use json::JsonValue;
use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserialize, Serialize, Serializer,
};
use tree_sitter::Node;

use std::{
    cmp::Ordering,
    convert::TryFrom,
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    ops::{Add, Sub},
};

pub mod function;
pub mod iter;
pub mod table;
pub mod time;
pub mod value_type;
pub mod variable_map;

/// Whale value representation.
///
/// Every whale variable has a key and a Value. Variables are represented by
/// storing them in a VariableMap. This means the map of variables is itself a
/// value that can be treated as any other.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum Value {
    String(String),
    Float(f64),
    Integer(i64),
    Boolean(bool),
    List(Vec<Value>),
    Map(VariableMap),
    Table(Table),
    Time(Time),
    Function(Function),
    #[default]
    Empty,
}

impl Value {
    pub fn new(node: Node, source: &str) -> Result<Self> {
        let node = if node.kind() == "value" {
            node.child(0).unwrap()
        } else {
            node
        };

        let value_snippet = &source[node.byte_range()];

        match node.kind() {
            "integer" => {
                let raw = value_snippet.parse::<i64>().unwrap_or_default();

                Ok(Value::Integer(raw))
            }
            "string" => {
                let quote_str = &value_snippet.chars().nth(0).unwrap();
                let without_quotes = value_snippet.trim_matches(*quote_str);

                Ok(Value::String(without_quotes.to_string()))
            }
            "boolean" => {
                let raw = value_snippet.parse::<bool>().unwrap_or_default();

                Ok(Value::Boolean(raw))
            }
            "float" => {
                let raw = value_snippet.parse::<f64>().unwrap_or_default();

                Ok(Value::Float(raw))
            }
            "list" => {
                let child_count = node.child_count();
                let mut values = Vec::with_capacity(child_count);

                // Skip the first and last nodes because they are parentheses.
                for index in 1..child_count - 1 {
                    let child = node.child(index).unwrap();

                    if child.kind() == "," {
                        continue;
                    }

                    let value = Value::new(child, source)?;

                    values.push(value)
                }

                Ok(Value::List(values))
            }
            "table" => {
                let child_count = node.child_count();
                let mut column_names = Vec::new();
                let mut rows = Vec::new();

                // Skip the first and last nodes because they are pointy braces.
                for index in 1..child_count - 1 {
                    let child = node.child(index).unwrap();

                    if child.kind() == "identifier" {
                        let child_expression = Expression::new(child, source)?;

                        if let Expression::Identifier(column_name) = child_expression {
                            column_names.push(column_name)
                        }
                    }

                    if child.kind() == "list" {
                        let child_value = Value::new(child, source)?;

                        if let Value::List(row) = child_value {
                            rows.push(row);
                        }
                    }
                }

                let mut table = Table::new(column_names);
                table.reserve(rows.len());

                for row in rows {
                    table.insert(row)?;
                }

                Ok(Value::Table(table))
            }
            "map" => {
                let child_count = node.child_count();
                let mut map = VariableMap::new();
                let mut key = String::new();

                for index in 0..child_count {
                    let child = node.child(index).unwrap();

                    if child.kind() == "identifier" {
                        let child_identifier = Expression::new(child, source)?;

                        if let Expression::Identifier(identifier) = child_identifier {
                            key = identifier
                        }
                    }

                    if child.kind() == "value" {
                        let value = Value::new(child, source)?;

                        map.set_value(key.as_str(), value)?;
                    }
                }

                Ok(Value::Map(map))
            }
            "function" => {
                let child_count = node.child_count();
                let mut identifiers = Vec::new();
                let mut statements = Vec::new();

                for index in 0..child_count {
                    let child = node.child(index).unwrap();

                    if child.kind() == "identifier" {
                        let child_identifier = Expression::new(child, source)?;

                        if let Expression::Identifier(identifier) = child_identifier {
                            identifiers.push(identifier)
                        }
                    }

                    if child.kind() == "statement" {
                        let statement = Statement::new(child, source)?;

                        statements.push(statement)
                    }
                }

                Ok(Value::Function(Function::new(identifiers, statements)))
            }
            "empty" => Ok(Value::Empty),
            _ => Err(Error::UnexpectedSourceNode {
                expected: "integer, string, boolean, float, list or empty",
                actual: node.kind(),
            }),
        }
    }

    pub fn value_type(&self) -> ValueType {
        ValueType::from(self)
    }

    pub fn is_table(&self) -> bool {
        matches!(self, Value::Table(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Value::Integer(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Value::Integer(_) | Value::Float(_))
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_))
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Value::Empty)
    }

    pub fn is_map(&self) -> bool {
        matches!(self, Value::Map(_))
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Value::Map(_))
    }

    /// Borrows the value stored in `self` as `String`, or returns `Err` if `self` is not a `Value::String`.
    pub fn as_string(&self) -> Result<&String> {
        match self {
            Value::String(string) => Ok(string),
            value => Err(Error::expected_string(value.clone())),
        }
    }

    /// Copies the value stored in `self` as `i64`, or returns `Err` if `self` is not a `Value::Int`.
    pub fn as_int(&self) -> Result<i64> {
        match self {
            Value::Integer(i) => Ok(*i),
            value => Err(Error::expected_int(value.clone())),
        }
    }

    /// Copies the value stored in  `self` as `f64`, or returns `Err` if `self` is not a `Value::Float`.
    pub fn as_float(&self) -> Result<f64> {
        match self {
            Value::Float(f) => Ok(*f),
            value => Err(Error::expected_float(value.clone())),
        }
    }

    /// Copies the value stored in  `self` as `f64`, or returns `Err` if `self` is not a `Value::Float` or `Value::Int`.
    /// Note that this method silently converts `i64` to `f64`, if `self` is a `Value::Int`.
    pub fn as_number(&self) -> Result<f64> {
        match self {
            Value::Float(f) => Ok(*f),
            Value::Integer(i) => Ok(*i as f64),
            value => Err(Error::expected_number(value.clone())),
        }
    }

    /// Copies the value stored in  `self` as `bool`, or returns `Err` if `self` is not a `Value::Boolean`.
    pub fn as_boolean(&self) -> Result<bool> {
        match self {
            Value::Boolean(boolean) => Ok(*boolean),
            value => Err(Error::expected_boolean(value.clone())),
        }
    }

    /// Borrows the value stored in `self` as `Vec<Value>`, or returns `Err` if `self` is not a `Value::List`.
    pub fn as_list(&self) -> Result<&Vec<Value>> {
        match self {
            Value::List(list) => Ok(list),
            value => Err(Error::expected_list(value.clone())),
        }
    }

    /// Borrows the value stored in `self` as `Vec<Value>`, or returns `Err` if `self` is not a `Value::List`.
    pub fn into_inner_list(self) -> Result<Vec<Value>> {
        match self {
            Value::List(list) => Ok(list),
            value => Err(Error::expected_list(value.clone())),
        }
    }

    /// Borrows the value stored in `self` as `Vec<Value>` or returns `Err` if `self` is not a `Value::Map` of the required length.
    pub fn as_fixed_len_list(&self, len: usize) -> Result<&Vec<Value>> {
        match self {
            Value::List(tuple) => {
                if tuple.len() == len {
                    Ok(tuple)
                } else {
                    Err(Error::expected_fixed_len_list(len, self.clone()))
                }
            }
            value => Err(Error::expected_list(value.clone())),
        }
    }

    /// Borrows the value stored in `self` as `Vec<Value>`, or returns `Err` if `self` is not a `Value::Map`.
    pub fn as_map(&self) -> Result<&VariableMap> {
        match self {
            Value::Map(map) => Ok(map),
            value => Err(Error::expected_map(value.clone())),
        }
    }

    /// Borrows the value stored in `self` as `Vec<Value>`, or returns `Err` if `self` is not a `Value::Table`.
    pub fn as_table(&self) -> Result<&Table> {
        match self {
            Value::Table(table) => Ok(table),
            value => Err(Error::expected_table(value.clone())),
        }
    }

    /// Borrows the value stored in `self` as `Function`, or returns `Err` if
    /// `self` is not a `Value::Function`.
    pub fn as_function(&self) -> Result<&Function> {
        match self {
            Value::Function(function) => Ok(function),
            value => Err(Error::expected_function(value.clone())),
        }
    }

    /// Borrows the value stored in `self` as `Time`, or returns `Err` if
    /// `self` is not a `Value::Time`.
    pub fn as_time(&self) -> Result<&Time> {
        match self {
            Value::Time(time) => Ok(time),
            value => Err(Error::expected_function(value.clone())),
        }
    }

    /// Returns `()`, or returns`Err` if `self` is not a `Value::Tuple`.
    pub fn as_empty(&self) -> Result<()> {
        match self {
            Value::Empty => Ok(()),
            value => Err(Error::expected_empty(value.clone())),
        }
    }

    /// Returns an owned table, either by cloning or converting the inner value..
    pub fn to_table(&self) -> Result<Table> {
        match self {
            Value::Table(table) => Ok(table.clone()),
            Value::List(list) => Ok(Table::from(list)),
            Value::Map(map) => Ok(Table::from(map)),
            value => Err(Error::expected_table(value.clone())),
        }
    }
}

impl Add for Value {
    type Output = Result<Value>;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Value::String(left), Value::String(right)) => {
                let concatenated = left + &right;

                Ok(Value::String(concatenated))
            }
            (Value::String(_), other) | (other, Value::String(_)) => {
                Err(Error::ExpectedString { actual: other })
            }
            (Value::Float(left), Value::Float(right)) => {
                let addition = left + right;

                Ok(Value::Float(addition))
            }
            (Value::Float(_), other) | (other, Value::Float(_)) => {
                Err(Error::ExpectedFloat { actual: other })
            }
            (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(left + right)),
            (Value::Integer(_), other) | (other, Value::Integer(_)) => {
                Err(Error::ExpectedInt { actual: other })
            }
            (Value::Boolean(_), Value::Boolean(_)) => todo!(),
            (Value::Boolean(_), other) | (other, Value::Boolean(_)) => {
                Err(Error::ExpectedBoolean { actual: other })
            }
            (Value::List(_), Value::List(_)) => todo!(),
            (Value::List(_), other) | (other, Value::List(_)) => {
                Err(Error::ExpectedList { actual: other })
            }
            (Value::Map(_), Value::Map(_)) => todo!(),
            (Value::Map(_), other) | (other, Value::Map(_)) => {
                Err(Error::ExpectedMap { actual: other })
            }
            (Value::Empty, Value::Empty) => Ok(Value::Empty),
            _ => Err(Error::CustomMessage(
                "Cannot add the given types.".to_string(),
            )),
        }
    }
}

impl Sub for Value {
    type Output = Result<Self>;

    fn sub(self, other: Self) -> Self::Output {
        match (&self, &other) {
            (Value::String(_), Value::String(_)) => Err(Error::ExpectedNumber {
                actual: self.clone(),
            }),
            (Value::String(_), other) | (other, Value::String(_)) => Err(Error::ExpectedNumber {
                actual: other.clone(),
            }),
            (Value::Float(left), Value::Float(right)) => {
                let addition = left - right;

                Ok(Value::Float(addition))
            }
            (Value::Float(_), other) | (other, Value::Float(_)) => Err(Error::ExpectedNumber {
                actual: other.clone(),
            }),
            (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(left - right)),
            (Value::Integer(_), other) | (other, Value::Integer(_)) => Err(Error::ExpectedInt {
                actual: other.clone(),
            }),
            (Value::Boolean(_), Value::Boolean(_)) => todo!(),
            (Value::Boolean(_), other) | (other, Value::Boolean(_)) => {
                Err(Error::ExpectedBoolean {
                    actual: other.clone(),
                })
            }
            (Value::List(_), Value::List(_)) => todo!(),
            (Value::List(_), other) | (other, Value::List(_)) => Err(Error::ExpectedList {
                actual: other.clone(),
            }),
            (Value::Map(_), Value::Map(_)) => todo!(),
            (Value::Map(_), other) | (other, Value::Map(_)) => Err(Error::ExpectedMap {
                actual: other.clone(),
            }),
            (Value::Empty, Value::Empty) => Ok(Value::Empty),
            _ => Err(Error::CustomMessage(
                "Cannot add the given types.".to_string(),
            )),
        }
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Value::String(left), Value::String(right)) => left.cmp(right),
            (Value::String(_), _) => Ordering::Greater,
            (Value::Integer(left), Value::Integer(right)) => left.cmp(right),
            (Value::Integer(_), _) => Ordering::Greater,
            (Value::Boolean(left), Value::Boolean(right)) => left.cmp(right),
            (Value::Boolean(_), _) => Ordering::Greater,
            (Value::Float(left), Value::Float(right)) => left.total_cmp(right),
            (Value::Float(_), _) => Ordering::Greater,
            (Value::List(left), Value::List(right)) => left.cmp(right),
            (Value::List(_), _) => Ordering::Greater,
            (Value::Map(left), Value::Map(right)) => left.cmp(right),
            (Value::Map(_), _) => Ordering::Greater,
            (Value::Table(left), Value::Table(right)) => left.cmp(right),
            (Value::Table(_), _) => Ordering::Greater,
            (Value::Function(left), Value::Function(right)) => left.cmp(right),
            (Value::Function(_), _) => Ordering::Greater,
            (Value::Time(left), Value::Time(right)) => left.cmp(right),
            (Value::Time(_), _) => Ordering::Greater,
            (Value::Empty, Value::Empty) => Ordering::Equal,
            (Value::Empty, _) => Ordering::Less,
        }
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::String(inner) => serializer.serialize_str(inner),
            Value::Float(inner) => serializer.serialize_f64(*inner),
            Value::Integer(inner) => serializer.serialize_i64(*inner),
            Value::Boolean(inner) => serializer.serialize_bool(*inner),
            Value::List(inner) => {
                let mut tuple = serializer.serialize_tuple(inner.len())?;

                for value in inner {
                    tuple.serialize_element(value)?;
                }

                tuple.end()
            }
            Value::Empty => todo!(),
            Value::Map(inner) => inner.serialize(serializer),
            Value::Table(inner) => inner.serialize(serializer),
            Value::Function(inner) => inner.serialize(serializer),
            Value::Time(inner) => inner.serialize(serializer),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::String(string) => write!(f, "{string}"),
            Value::Float(float) => write!(f, "{}", float),
            Value::Integer(int) => write!(f, "{}", int),
            Value::Boolean(boolean) => write!(f, "{}", boolean),
            Value::Empty => write!(f, "()"),
            Value::List(list) => Table::from(list).fmt(f),
            Value::Map(map) => write!(f, "{map}"),
            Value::Table(table) => write!(f, "{table}"),
            Value::Function(function) => write!(f, "{function}"),
            Value::Time(time) => write!(f, "{time}"),
        }
    }
}

impl From<String> for Value {
    fn from(string: String) -> Self {
        Value::String(string)
    }
}

impl From<&str> for Value {
    fn from(string: &str) -> Self {
        Value::String(string.to_string())
    }
}

impl From<f64> for Value {
    fn from(float: f64) -> Self {
        Value::Float(float)
    }
}

impl From<i64> for Value {
    fn from(int: i64) -> Self {
        Value::Integer(int)
    }
}

impl From<bool> for Value {
    fn from(boolean: bool) -> Self {
        Value::Boolean(boolean)
    }
}

impl From<Vec<Value>> for Value {
    fn from(tuple: Vec<Value>) -> Self {
        Value::List(tuple)
    }
}

impl From<Value> for Result<Value> {
    fn from(value: Value) -> Self {
        Ok(value)
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Empty
    }
}

impl TryFrom<JsonValue> for Value {
    type Error = Error;

    fn try_from(json_value: JsonValue) -> Result<Self> {
        use JsonValue::*;

        match json_value {
            Null => Ok(Value::Empty),
            Short(short) => Ok(Value::String(short.to_string())),
            String(string) => Ok(Value::String(string)),
            Number(number) => Ok(Value::Float(f64::from(number))),
            Boolean(boolean) => Ok(Value::Boolean(boolean)),
            Object(object) => {
                let mut map = VariableMap::new();

                for (key, node_value) in object.iter() {
                    let value = Value::try_from(node_value)?;

                    map.set_value(key, value)?;
                }

                Ok(Value::Map(map))
            }
            Array(array) => {
                let mut list = Vec::new();

                for json_value in array {
                    let value = Value::try_from(json_value)?;

                    list.push(value);
                }

                Ok(Value::List(list))
            }
        }
    }
}

impl TryFrom<&JsonValue> for Value {
    type Error = Error;

    fn try_from(json_value: &JsonValue) -> Result<Self> {
        use JsonValue::*;

        match json_value {
            Null => Ok(Value::Empty),
            Short(short) => Ok(Value::String(short.to_string())),
            String(string) => Ok(Value::String(string.clone())),
            Number(number) => Ok(Value::Float(f64::from(*number))),
            Boolean(boolean) => Ok(Value::Boolean(*boolean)),
            Object(object) => {
                let mut map = VariableMap::new();

                for (key, node_value) in object.iter() {
                    let value = Value::try_from(node_value)?;

                    map.set_value(key, value)?;
                }

                Ok(Value::Map(map))
            }
            Array(array) => {
                let mut list = Vec::new();

                for json_value in array {
                    let value = Value::try_from(json_value)?;

                    list.push(value);
                }

                Ok(Value::List(list))
            }
        }
    }
}

impl TryFrom<Value> for String {
    type Error = Error;

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        if let Value::String(value) = value {
            Ok(value)
        } else {
            Err(Error::ExpectedString { actual: value })
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = Error;

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        if let Value::Float(value) = value {
            Ok(value)
        } else {
            Err(Error::ExpectedFloat { actual: value })
        }
    }
}

impl TryFrom<Value> for i64 {
    type Error = Error;

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        if let Value::Integer(value) = value {
            Ok(value)
        } else {
            Err(Error::ExpectedInt { actual: value })
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = Error;

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        if let Value::Boolean(value) = value {
            Ok(value)
        } else {
            Err(Error::ExpectedBoolean { actual: value })
        }
    }
}

struct ValueVisitor {
    marker: PhantomData<fn() -> Value>,
}

impl ValueVisitor {
    fn new() -> Self {
        ValueVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("Any valid whale data.")
    }

    fn visit_bool<E>(self, v: bool) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Boolean(v))
    }

    fn visit_i8<E>(self, v: i8) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_i16<E>(self, v: i16) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_i32<E>(self, v: i32) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(v))
    }

    fn visit_i128<E>(self, v: i128) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v > i64::MAX as i128 {
            Ok(Value::Integer(i64::MAX))
        } else {
            Ok(Value::Integer(v as i64))
        }
    }

    fn visit_u8<E>(self, v: u8) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u64(v as u64)
    }

    fn visit_u16<E>(self, v: u16) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u64(v as u64)
    }

    fn visit_u32<E>(self, v: u32) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u64(v as u64)
    }

    fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_u128<E>(self, v: u128) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i128(v as i128)
    }

    fn visit_f32<E>(self, v: f32) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_f64(v as f64)
    }

    fn visit_f64<E>(self, v: f64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Float(v))
    }

    fn visit_char<E>(self, v: char) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&v.to_string())
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::String(v.to_string()))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v)
    }

    fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::String(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let _ = v;
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Bytes(v),
            &self,
        ))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_bytes(v)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_bytes(&v)
    }

    fn visit_none<E>(self) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Option,
            &self,
        ))
    }

    fn visit_some<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let _ = deserializer;
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Option,
            &self,
        ))
    }

    fn visit_unit<E>(self) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Unit,
            &self,
        ))
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let _ = deserializer;
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::NewtypeStruct,
            &self,
        ))
    }

    fn visit_seq<A>(self, mut access: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut list = Vec::new();

        while let Some(value) = access.next_element()? {
            list.push(value);
        }

        Ok(Value::List(list))
    }

    fn visit_map<M>(self, mut access: M) -> std::result::Result<Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = VariableMap::new();

        while let Some((key, value)) = access.next_entry()? {
            map.set_value(key, value)
                .expect("Failed to deserialize Value. This is a no-op.");
        }

        Ok(Value::Map(map))
    }

    fn visit_enum<A>(self, data: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        let _ = data;
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Enum,
            &self,
        ))
    }

    fn __private_visit_untagged_option<D>(self, _: D) -> std::result::Result<Self::Value, ()>
    where
        D: serde::Deserializer<'de>,
    {
        Err(())
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor::new())
    }
}
