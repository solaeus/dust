//! Types that represent runtime values.
use crate::{
    error::{Error, Result},
    Function, List, Map, Type, TypeDefinition,
};

use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserialize, Serialize, Serializer,
};

use std::{
    cmp::Ordering,
    convert::TryFrom,
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    ops::{Add, AddAssign, Div, Mul, Rem, Sub, SubAssign},
};

pub mod function;
pub mod list;
pub mod map;
pub mod table;

/// Dust value representation.
///
/// Every dust variable has a key and a Value. Variables are represented by
/// storing them in a VariableMap. This means the map of variables is itself a
/// value that can be treated as any other.
#[derive(Debug, Clone, Default)]
pub enum Value {
    List(List),
    Map(Map),
    Function(Function),
    String(String),
    Float(f64),
    Integer(i64),
    Boolean(bool),
    #[default]
    Empty,
}

impl Value {
    pub fn r#type(&self, context: &Map) -> Result<TypeDefinition> {
        let r#type = match self {
            Value::List(list) => {
                let mut previous_type = None;

                for value in list.items().iter() {
                    let value_type = value.r#type(context)?;

                    if let Some(previous) = &previous_type {
                        if &value_type != previous {
                            return Ok(TypeDefinition::new(Type::List(Box::new(Type::Any))));
                        }
                    }

                    previous_type = Some(value_type);
                }

                if let Some(previous) = previous_type {
                    Type::List(Box::new(previous.take_inner()))
                } else {
                    Type::List(Box::new(Type::Any))
                }
            }
            Value::Map(_) => Type::Map,
            Value::Function(function) => return Ok(TypeDefinition::new(function.r#type().clone())),
            Value::String(_) => Type::String,
            Value::Float(_) => Type::Float,
            Value::Integer(_) => Type::Integer,
            Value::Boolean(_) => Type::Boolean,
            Value::Empty => Type::Empty,
        };

        Ok(TypeDefinition::new(r#type))
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
            value => Err(Error::ExpectedString {
                actual: value.clone(),
            }),
        }
    }

    /// Copies the value stored in `self` as `i64`, or returns `Err` if `self` is not a `Value::Int`.
    pub fn as_integer(&self) -> Result<i64> {
        match self {
            Value::Integer(i) => Ok(*i),
            value => Err(Error::ExpectedInteger {
                actual: value.clone(),
            }),
        }
    }

    /// Copies the value stored in  `self` as `f64`, or returns `Err` if `self` is not a `Primitive::Float`.
    pub fn as_float(&self) -> Result<f64> {
        match self {
            Value::Float(f) => Ok(*f),
            value => Err(Error::ExpectedFloat {
                actual: value.clone(),
            }),
        }
    }

    /// Copies the value stored in  `self` as `f64`, or returns `Err` if `self` is not a `Primitive::Float` or `Value::Int`.
    /// Note that this method silently converts `i64` to `f64`, if `self` is a `Value::Int`.
    pub fn as_number(&self) -> Result<f64> {
        match self {
            Value::Float(f) => Ok(*f),
            Value::Integer(i) => Ok(*i as f64),
            value => Err(Error::ExpectedNumber {
                actual: value.clone(),
            }),
        }
    }

    /// Copies the value stored in  `self` as `bool`, or returns `Err` if `self` is not a `Primitive::Boolean`.
    pub fn as_boolean(&self) -> Result<bool> {
        match self {
            Value::Boolean(boolean) => Ok(*boolean),
            value => Err(Error::ExpectedBoolean {
                actual: value.clone(),
            }),
        }
    }

    /// Borrows the value stored in `self` as `Vec<Value>`, or returns `Err` if `self` is not a `Value::List`.
    pub fn as_list(&self) -> Result<&List> {
        match self {
            Value::List(list) => Ok(list),
            value => Err(Error::ExpectedList {
                actual: value.clone(),
            }),
        }
    }

    /// Borrows the value stored in `self` as `Vec<Value>`, or returns `Err` if `self` is not a `Value::List`.
    pub fn into_inner_list(self) -> Result<List> {
        match self {
            Value::List(list) => Ok(list),
            value => Err(Error::ExpectedList {
                actual: value.clone(),
            }),
        }
    }

    /// Borrows the value stored in `self` as `Vec<Value>`, or returns `Err` if `self` is not a `Value::Map`.
    pub fn as_map(&self) -> Result<&Map> {
        match self {
            Value::Map(map) => Ok(map),
            value => Err(Error::ExpectedMap {
                actual: value.clone(),
            }),
        }
    }

    /// Borrows the value stored in `self` as `Function`, or returns `Err` if
    /// `self` is not a `Value::Function`.
    pub fn as_function(&self) -> Result<&Function> {
        match self {
            Value::Function(function) => Ok(function),
            value => Err(Error::ExpectedFunction {
                actual: value.clone(),
            }),
        }
    }

    /// Returns `()`, or returns`Err` if `self` is not a `Value::Empty`.
    pub fn as_empty(&self) -> Result<()> {
        match self {
            Value::Empty => Ok(()),
            value => Err(Error::ExpectedEmpty {
                actual: value.clone(),
            }),
        }
    }
}

impl Default for &Value {
    fn default() -> Self {
        &Value::Empty
    }
}

impl Add for Value {
    type Output = Result<Value>;

    fn add(self, other: Self) -> Self::Output {
        if let (Ok(left), Ok(right)) = (self.as_integer(), other.as_integer()) {
            return Ok(Value::Integer(left + right));
        }

        if let (Ok(left), Ok(right)) = (self.as_number(), other.as_number()) {
            return Ok(Value::Float(left + right));
        }

        if let (Ok(left), Ok(right)) = (self.as_string(), other.as_string()) {
            return Ok(Value::String(left.to_string() + right));
        }

        if self.is_string() || other.is_string() {
            return Ok(Value::String(self.to_string() + &other.to_string()));
        }

        let non_number_or_string = if !self.is_number() == !self.is_string() {
            self
        } else {
            other
        };

        Err(Error::ExpectedNumberOrString {
            actual: non_number_or_string,
        })
    }
}

impl Sub for Value {
    type Output = Result<Self>;

    fn sub(self, other: Self) -> Self::Output {
        match (self.as_integer(), other.as_integer()) {
            (Ok(left), Ok(right)) => return Ok(Value::Integer(left - right)),
            _ => {}
        }

        match (self.as_number(), other.as_number()) {
            (Ok(left), Ok(right)) => return Ok(Value::Float(left - right)),
            _ => {}
        }

        let non_number = if !self.is_number() { self } else { other };

        Err(Error::ExpectedNumber { actual: non_number })
    }
}

impl Mul for Value {
    type Output = Result<Self>;

    fn mul(self, other: Self) -> Self::Output {
        if self.is_integer() && other.is_integer() {
            let left = self.as_integer().unwrap();
            let right = other.as_integer().unwrap();
            let value = Value::Integer(left.saturating_mul(right));

            Ok(value)
        } else {
            let left = self.as_number()?;
            let right = other.as_number()?;
            let value = Value::Float(left * right);

            Ok(value)
        }
    }
}

impl Div for Value {
    type Output = Result<Self>;

    fn div(self, other: Self) -> Self::Output {
        let left = self.as_number()?;
        let right = other.as_number()?;
        let result = left / right;
        let value = if result % 2.0 == 0.0 {
            Value::Integer(result as i64)
        } else {
            Value::Float(result)
        };

        Ok(value)
    }
}

impl Rem for Value {
    type Output = Result<Self>;

    fn rem(self, other: Self) -> Self::Output {
        let left = self.as_integer()?;
        let right = other.as_integer()?;
        let result = left % right;

        Ok(Value::Integer(result))
    }
}

impl AddAssign for Value {
    fn add_assign(&mut self, other: Self) {
        match (self, other) {
            (Value::Integer(left), Value::Integer(right)) => *left += right,
            (Value::Float(left), Value::Float(right)) => *left += right,
            (Value::Float(left), Value::Integer(right)) => *left += right as f64,
            (Value::String(left), Value::String(right)) => *left += &right,
            (Value::List(list), value) => list.items_mut().push(value),
            _ => {}
        }
    }
}

impl SubAssign for Value {
    fn sub_assign(&mut self, other: Self) {
        match (self, other) {
            (Value::Integer(left), Value::Integer(right)) => *left -= right,
            (Value::Float(left), Value::Float(right)) => *left -= right,
            (Value::Float(left), Value::Integer(right)) => *left -= right as f64,
            (Value::List(list), value) => {
                let index_to_remove = list
                    .items()
                    .iter()
                    .enumerate()
                    .find_map(|(i, list_value)| if list_value == &value { Some(i) } else { None });

                if let Some(index) = index_to_remove {
                    list.items_mut().remove(index);
                }
            }
            _ => {}
        }
    }
}

impl Eq for Value {}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(left), Value::Integer(right)) => left == right,
            (Value::Float(left), Value::Float(right)) => left == right,
            (Value::Boolean(left), Value::Boolean(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::List(left), Value::List(right)) => left == right,
            (Value::Map(left), Value::Map(right)) => left == right,
            (Value::Function(left), Value::Function(right)) => left == right,
            (Value::Empty, Value::Empty) => true,
            _ => false,
        }
    }
}

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
            (Value::Float(left), Value::Float(right)) => left.total_cmp(right),
            (Value::Integer(left), Value::Integer(right)) => left.cmp(right),
            (Value::Float(float), Value::Integer(integer)) => {
                let int_as_float = *integer as f64;
                float.total_cmp(&int_as_float)
            }
            (Value::Integer(integer), Value::Float(float)) => {
                let int_as_float = *integer as f64;
                int_as_float.total_cmp(float)
            }
            (Value::Float(_), _) => Ordering::Greater,
            (Value::Integer(_), _) => Ordering::Greater,
            (Value::Boolean(left), Value::Boolean(right)) => left.cmp(right),
            (Value::Boolean(_), _) => Ordering::Greater,
            (Value::List(left), Value::List(right)) => left.cmp(right),
            (Value::List(_), _) => Ordering::Greater,
            (Value::Map(left), Value::Map(right)) => left.cmp(right),
            (Value::Map(_), _) => Ordering::Greater,
            (Value::Function(left), Value::Function(right)) => left.cmp(right),
            (Value::Function(_), _) => Ordering::Greater,
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
                let items = inner.items();
                let mut list = serializer.serialize_tuple(items.len())?;

                for value in items.iter() {
                    list.serialize_element(value)?;
                }

                list.end()
            }
            Value::Empty => todo!(),
            Value::Map(inner) => inner.serialize(serializer),
            Value::Function(inner) => inner.serialize(serializer),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::String(string) => write!(f, "{string}"),
            Value::Float(float) => write!(f, "{float}"),
            Value::Integer(int) => write!(f, "{int}"),
            Value::Boolean(boolean) => write!(f, "{boolean}"),
            Value::Empty => write!(f, "empty"),
            Value::List(list) => {
                write!(f, "[")?;
                for value in list.items().iter() {
                    write!(f, " {value} ")?;
                }
                write!(f, "]")
            }
            Value::Map(map) => write!(f, "{map}"),
            Value::Function(function) => write!(f, "{function}"),
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
    fn from(vec: Vec<Value>) -> Self {
        Value::List(List::with_items(vec))
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
            Err(Error::ExpectedInteger { actual: value })
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
        Ok(Value::Empty)
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
        Ok(Value::Empty)
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

        Ok(Value::List(List::with_items(list)))
    }

    fn visit_map<M>(self, mut access: M) -> std::result::Result<Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let map = Map::new();

        {
            let mut variables = map.variables_mut().unwrap();

            while let Some((key, value)) = access.next_entry()? {
                variables.insert(key, value);
            }
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
