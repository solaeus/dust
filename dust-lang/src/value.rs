//! Dust value representation
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    error::Error,
    fmt::{self, Display, Formatter},
    ops::Range,
    sync::{Arc, RwLock},
};

use serde::{
    de::Visitor,
    ser::{SerializeMap, SerializeSeq, SerializeTuple},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{AbstractSyntaxTree, Context, Identifier, StructType, Type, Vm, VmError};

/// Dust value representation
///
/// Each type of value has a corresponding constructor, here are some simple examples:
///
/// ```
/// # use dust_lang::Value;
/// let boolean = Value::boolean(true);
/// let float = Value::float(3.14);
/// let integer = Value::integer(42);
/// let string = Value::string("Hello, world!");
/// ```
///
/// Values can be combined into more complex values:
///
/// ```
/// # use dust_lang::Value;
/// let list = Value::list(vec![
///     Value::integer(1),
///     Value::integer(2),
///     Value::integer(3),
/// ]);
/// ```
///
/// Values have a type, which can be retrieved using the `type` method:
///
/// ```
/// # use std::collections::HashMap;
/// # use dust_lang::*;
/// let value = Value::integer(42);
///
/// assert_eq!(value.r#type(), Type::Integer);
/// ```
#[derive(Clone, Debug)]
pub enum Value {
    Immutable(Arc<ValueData>),
    Mutable(Arc<RwLock<ValueData>>),
}

impl Value {
    pub fn boolean(boolean: bool) -> Self {
        Value::Immutable(Arc::new(ValueData::Boolean(boolean)))
    }

    pub fn float(float: f64) -> Self {
        Value::Immutable(Arc::new(ValueData::Float(float)))
    }

    pub fn function(function: Function) -> Self {
        Value::Immutable(Arc::new(ValueData::Function(function)))
    }

    pub fn integer(integer: i64) -> Self {
        Value::Immutable(Arc::new(ValueData::Integer(integer)))
    }

    pub fn list(list: Vec<Value>) -> Self {
        Value::Immutable(Arc::new(ValueData::List(list)))
    }

    pub fn map(map: BTreeMap<Identifier, Value>) -> Self {
        Value::Immutable(Arc::new(ValueData::Map(map)))
    }

    pub fn range(range: Range<i64>) -> Self {
        Value::Immutable(Arc::new(ValueData::Range(range)))
    }

    pub fn string<T: ToString>(to_string: T) -> Self {
        Value::Immutable(Arc::new(ValueData::String(to_string.to_string())))
    }

    pub fn r#struct(r#struct: Struct) -> Self {
        Value::Immutable(Arc::new(ValueData::Struct(r#struct)))
    }

    pub fn boolean_mut(boolean: bool) -> Self {
        Value::Mutable(Arc::new(RwLock::new(ValueData::Boolean(boolean))))
    }

    pub fn string_mut<T: ToString>(to_string: T) -> Self {
        Value::Mutable(Arc::new(RwLock::new(ValueData::String(
            to_string.to_string(),
        ))))
    }

    pub fn is_mutable(&self) -> bool {
        matches!(self, Value::Mutable(_))
    }

    pub fn to_mut(self) -> Self {
        match self {
            Value::Immutable(inner) => {
                Value::Mutable(Arc::new(RwLock::new(inner.as_ref().clone())))
            }
            _ => self,
        }
    }

    pub fn mutate(&self, other: &Value) {
        let other_data = match other {
            Value::Immutable(inner) => inner.as_ref().clone(),
            Value::Mutable(inner_locked) => inner_locked.read().unwrap().clone(),
        };

        match self {
            Value::Mutable(locked) => {
                *locked.write().unwrap() = other_data;
            }
            Value::Immutable(_) => todo!(),
        }
    }

    pub fn r#type(&self) -> Type {
        match self {
            Value::Immutable(inner) => inner.r#type(),
            Value::Mutable(inner_locked) => inner_locked.read().unwrap().r#type(),
        }
    }

    pub fn get_property(&self, property: &Identifier) -> Option<Value> {
        match self {
            Value::Immutable(inner) => inner.get_property(property),
            Value::Mutable(inner_locked) => inner_locked.read().unwrap().get_property(property),
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Immutable(arc) => match arc.as_ref() {
                ValueData::Boolean(boolean) => Some(*boolean),
                _ => None,
            },
            Value::Mutable(arc_rw_lock) => match *arc_rw_lock.read().unwrap() {
                ValueData::Boolean(boolean) => Some(boolean),
                _ => None,
            },
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Immutable(arc) => match arc.as_ref() {
                ValueData::Float(float) => Some(*float),
                _ => None,
            },
            Value::Mutable(arc_rw_lock) => match *arc_rw_lock.read().unwrap() {
                ValueData::Float(float) => Some(float),
                _ => None,
            },
        }
    }

    pub fn as_function(&self) -> Option<&Function> {
        if let Value::Immutable(arc) = self {
            if let ValueData::Function(function) = arc.as_ref() {
                return Some(function);
            }
        }

        None
    }

    pub fn as_list(&self) -> Option<&Vec<Value>> {
        if let Value::Immutable(arc) = self {
            if let ValueData::List(list) = arc.as_ref() {
                return Some(list);
            }
        }

        None
    }

    pub fn as_map(&self) -> Option<&BTreeMap<Identifier, Value>> {
        if let Value::Immutable(arc) = self {
            if let ValueData::Map(map) = arc.as_ref() {
                return Some(map);
            }
        }

        None
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Immutable(arc) => match arc.as_ref() {
                ValueData::Integer(integer) => Some(*integer),
                _ => None,
            },
            Value::Mutable(arc_rw_lock) => match *arc_rw_lock.read().unwrap() {
                ValueData::Integer(integer) => Some(integer),
                _ => None,
            },
        }
    }

    pub fn as_range(&self) -> Option<&Range<i64>> {
        if let Value::Immutable(arc) = self {
            if let ValueData::Range(range) = arc.as_ref() {
                return Some(range);
            }
        }

        None
    }

    pub fn as_string(&self) -> Option<&String> {
        if let Value::Immutable(arc) = self {
            if let ValueData::String(string) = arc.as_ref() {
                return Some(string);
            }
        }

        None
    }

    pub fn add(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Immutable(left), Value::Immutable(right)) => {
                match (left.as_ref(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::float(left + right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::integer(left.saturating_add(*right)));
                    }
                    (ValueData::String(left), ValueData::String(right)) => {
                        return Ok(Value::string(left.to_string() + right));
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                match (&*left.read().unwrap(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::float(left + right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::integer(left.saturating_add(*right)));
                    }
                    (ValueData::String(left), ValueData::String(right)) => {
                        return Ok(Value::string(left.to_string() + right));
                    }
                    _ => {}
                }
            }
            (Value::Immutable(left), Value::Mutable(right)) => {
                match (left.as_ref(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::float(left + right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::integer(left.saturating_add(*right)));
                    }
                    (ValueData::String(left), ValueData::String(right)) => {
                        return Ok(Value::string(left.to_string() + right));
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Immutable(right)) => {
                match (&*left.read().unwrap(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::float(left + right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::integer(left.saturating_add(*right)));
                    }
                    (ValueData::String(left), ValueData::String(right)) => {
                        return Ok(Value::string(left.to_string() + right));
                    }
                    _ => {}
                }
            }
        }

        Err(ValueError::CannotAdd(self.clone(), other.clone()))
    }

    pub fn add_mut(&self, other: &Value) -> Result<(), ValueError> {
        match (self, other) {
            (Value::Mutable(left), Value::Mutable(right)) => {
                match (&mut *left.write().unwrap(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        *left += right;
                        return Ok(());
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        *left = left.saturating_add(*right);
                        return Ok(());
                    }
                    (ValueData::String(left), ValueData::String(right)) => {
                        left.push_str(right);
                        return Ok(());
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Immutable(right)) => {
                match (&mut *left.write().unwrap(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        *left += right;
                        return Ok(());
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        *left = left.saturating_add(*right);
                        return Ok(());
                    }
                    (ValueData::String(left), ValueData::String(right)) => {
                        left.push_str(right);
                        return Ok(());
                    }
                    _ => {}
                }
            }
            (Value::Immutable(_), _) => {
                return Err(ValueError::CannotMutate(self.clone()));
            }
        }

        Err(ValueError::CannotAdd(self.clone(), other.clone()))
    }

    pub fn subtract(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Immutable(left), Value::Immutable(right)) => {
                match (left.as_ref(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::float(left - right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::integer(left.saturating_sub(*right)));
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                match (&*left.read().unwrap(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::float(left - right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::integer(left.saturating_sub(*right)));
                    }
                    _ => {}
                }
            }
            (Value::Immutable(left), Value::Mutable(right)) => {
                match (left.as_ref(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::float(left - right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::integer(left.saturating_sub(*right)));
                    }
                    _ => {}
                }
            }
            (Value::Mutable(right), Value::Immutable(left)) => {
                match (&*right.read().unwrap(), left.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::float(left - right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::integer(left.saturating_sub(*right)));
                    }
                    _ => {}
                }
            }
        }

        Err(ValueError::CannotSubtract(self.clone(), other.clone()))
    }

    pub fn subtract_mut(&self, other: &Value) -> Result<(), ValueError> {
        match (self, other) {
            (Value::Mutable(left), Value::Mutable(right)) => {
                match (&mut *left.write().unwrap(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        *left -= right;
                        return Ok(());
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        *left = left.saturating_sub(*right);
                        return Ok(());
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Immutable(right)) => {
                match (&mut *left.write().unwrap(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        *left -= right;
                        return Ok(());
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        *left = left.saturating_sub(*right);
                        return Ok(());
                    }
                    _ => {}
                }
            }
            (Value::Immutable(_), _) => {
                return Err(ValueError::CannotMutate(self.clone()));
            }
        }

        Err(ValueError::CannotSubtract(self.clone(), other.clone()))
    }

    pub fn multiply(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Immutable(left), Value::Immutable(right)) => {
                match (left.as_ref(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::float(left * right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::integer(left.saturating_mul(*right)));
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                match (&*left.read().unwrap(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::float(left * right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::integer(left.saturating_mul(*right)));
                    }
                    _ => {}
                }
            }
            (Value::Immutable(data), Value::Mutable(data_locked))
            | (Value::Mutable(data_locked), Value::Immutable(data)) => {
                match (&*data_locked.read().unwrap(), data.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::float(left * right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::integer(left.saturating_mul(*right)));
                    }
                    _ => {}
                }
            }
        }

        Err(ValueError::CannotMultiply(self.clone(), other.clone()))
    }

    pub fn divide(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Immutable(left), Value::Immutable(right)) => {
                match (left.as_ref(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        if *right == 0.0 {
                            return Err(ValueError::DivisionByZero);
                        }
                        return Ok(Value::float(left / right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        if *right == 0 {
                            return Err(ValueError::DivisionByZero);
                        }
                        return Ok(Value::integer(left / right));
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                match (&*left.read().unwrap(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        if *right == 0.0 {
                            return Err(ValueError::DivisionByZero);
                        }
                        return Ok(Value::float(left / right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        if *right == 0 {
                            return Err(ValueError::DivisionByZero);
                        }
                        return Ok(Value::integer(left / right));
                    }
                    _ => {}
                }
            }
            (Value::Immutable(left), Value::Mutable(right)) => {
                match (left.as_ref(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        if *right == 0.0 {
                            return Err(ValueError::DivisionByZero);
                        }
                        return Ok(Value::float(left / right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        if *right == 0 {
                            return Err(ValueError::DivisionByZero);
                        }
                        return Ok(Value::integer(left / right));
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Immutable(right)) => {
                match (&*left.read().unwrap(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        if *right == 0.0 {
                            return Err(ValueError::DivisionByZero);
                        }
                        return Ok(Value::float(left / right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        if *right == 0 {
                            return Err(ValueError::DivisionByZero);
                        }
                        return Ok(Value::integer(left / right));
                    }
                    _ => {}
                }
            }
        }

        Err(ValueError::CannotDivide(self.clone(), other.clone()))
    }

    pub fn modulo(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Immutable(left), Value::Immutable(right)) => {
                match (left.as_ref(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::float(left % right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        if *right == 0 {
                            return Err(ValueError::DivisionByZero);
                        }
                        return Ok(Value::integer(left % right));
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                match (&*left.read().unwrap(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::float(left % right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        if *right == 0 {
                            return Err(ValueError::DivisionByZero);
                        }
                        return Ok(Value::integer(left % right));
                    }
                    _ => {}
                }
            }
            (Value::Immutable(left), Value::Mutable(right)) => {
                match (left.as_ref(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::float(left % right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        if *right == 0 {
                            return Err(ValueError::DivisionByZero);
                        }
                        return Ok(Value::integer(left % right));
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Immutable(right)) => {
                match (&*left.read().unwrap(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::float(left % right));
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        if *right == 0 {
                            return Err(ValueError::DivisionByZero);
                        }
                        return Ok(Value::integer(left % right));
                    }
                    _ => {}
                }
            }
        }

        Err(ValueError::CannotModulo(self.clone(), other.clone()))
    }

    pub fn less_than(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Immutable(left), Value::Immutable(right)) => {
                match (left.as_ref(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::boolean(left < right))
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::boolean(left < right))
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                match (&*left.read().unwrap(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::boolean(left < right))
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::boolean(left < right))
                    }
                    _ => {}
                }
            }
            (Value::Immutable(left), Value::Mutable(right)) => {
                match (left.as_ref(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::boolean(left < right))
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::boolean(left < right))
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Immutable(right)) => {
                match (&*left.read().unwrap(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::boolean(left < right))
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::boolean(left < right))
                    }
                    _ => {}
                }
            }
        }

        Err(ValueError::CannotLessThan(self.clone(), other.clone()))
    }

    pub fn less_than_or_equal(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Immutable(left), Value::Immutable(right)) => {
                match (left.as_ref(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::boolean(left <= right))
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::boolean(left <= right))
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                match (&*left.read().unwrap(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::boolean(left <= right))
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::boolean(left <= right))
                    }
                    _ => {}
                }
            }
            (Value::Immutable(left), Value::Mutable(right)) => {
                match (left.as_ref(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::boolean(left <= right))
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::boolean(left <= right))
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Immutable(right)) => {
                match (&*left.read().unwrap(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::boolean(left <= right))
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::boolean(left <= right))
                    }
                    _ => {}
                }
            }
        }

        Err(ValueError::CannotLessThanOrEqual(
            self.clone(),
            other.clone(),
        ))
    }

    pub fn greater_than(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Immutable(left), Value::Immutable(right)) => {
                match (left.as_ref(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::boolean(left > right))
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::boolean(left > right))
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                match (&*left.read().unwrap(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::boolean(left > right))
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::boolean(left > right))
                    }
                    _ => {}
                }
            }
            (Value::Immutable(left), Value::Mutable(right)) => {
                match (left.as_ref(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::boolean(left > right))
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::boolean(left > right))
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Immutable(right)) => {
                match (&*left.read().unwrap(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::boolean(left > right))
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::boolean(left > right))
                    }
                    _ => {}
                }
            }
        }

        Err(ValueError::CannotGreaterThan(self.clone(), other.clone()))
    }

    pub fn greater_than_or_equal(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Immutable(left), Value::Immutable(right)) => {
                match (left.as_ref(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::boolean(left >= right))
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::boolean(left >= right))
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                match (&*left.read().unwrap(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::boolean(left >= right))
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::boolean(left >= right))
                    }
                    _ => {}
                }
            }
            (Value::Immutable(left), Value::Mutable(right)) => {
                match (left.as_ref(), &*right.read().unwrap()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::boolean(left >= right))
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::boolean(left >= right))
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), Value::Immutable(right)) => {
                match (&*left.read().unwrap(), right.as_ref()) {
                    (ValueData::Float(left), ValueData::Float(right)) => {
                        return Ok(Value::boolean(left >= right))
                    }
                    (ValueData::Integer(left), ValueData::Integer(right)) => {
                        return Ok(Value::boolean(left >= right))
                    }
                    _ => {}
                }
            }
        }

        Err(ValueError::CannotGreaterThanOrEqual(
            self.clone(),
            other.clone(),
        ))
    }

    pub fn and(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Immutable(left), Value::Immutable(right)) => {
                if let (ValueData::Boolean(left), ValueData::Boolean(right)) =
                    (left.as_ref(), right.as_ref())
                {
                    return Ok(Value::boolean(*left && *right));
                }
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                if let (ValueData::Boolean(left), ValueData::Boolean(right)) =
                    (&*left.read().unwrap(), &*right.read().unwrap())
                {
                    return Ok(Value::boolean(*left && *right));
                }
            }
            (Value::Mutable(locked), Value::Immutable(data))
            | (Value::Immutable(data), Value::Mutable(locked)) => {
                if let (ValueData::Boolean(left), ValueData::Boolean(right)) =
                    (&*locked.read().unwrap(), data.as_ref())
                {
                    return Ok(Value::boolean(*left && *right));
                }
            }
        }

        Err(ValueError::CannotAnd(self.clone(), other.clone()))
    }

    pub fn or(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Immutable(left), Value::Immutable(right)) => {
                if let (ValueData::Boolean(left), ValueData::Boolean(right)) =
                    (left.as_ref(), right.as_ref())
                {
                    return Ok(Value::boolean(*left || *right));
                }
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                if let (ValueData::Boolean(left), ValueData::Boolean(right)) =
                    (&*left.read().unwrap(), &*right.read().unwrap())
                {
                    return Ok(Value::boolean(*left || *right));
                }
            }
            (Value::Mutable(locked), Value::Immutable(data))
            | (Value::Immutable(data), Value::Mutable(locked)) => {
                if let (ValueData::Boolean(left), ValueData::Boolean(right)) =
                    (&*locked.read().unwrap(), data.as_ref())
                {
                    return Ok(Value::boolean(*left || *right));
                }
            }
        }

        Err(ValueError::CannotOr(self.clone(), other.clone()))
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Immutable(inner) => write!(f, "{inner}"),
            Value::Mutable(inner_locked) => {
                let inner = inner_locked.read().unwrap();

                write!(f, "{inner}")
            }
        }
    }
}

impl Eq for Value {}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Immutable(left), Value::Immutable(right)) => left == right,
            (Value::Mutable(left), Value::Mutable(right)) => {
                *left.read().unwrap() == *right.read().unwrap()
            }
            (Value::Immutable(inner), Value::Mutable(inner_locked))
            | (Value::Mutable(inner_locked), Value::Immutable(inner)) => {
                **inner == *inner_locked.read().unwrap()
            }
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Value::Immutable(left), Value::Immutable(right)) => left.cmp(right),
            (Value::Mutable(left), Value::Mutable(right)) => {
                left.read().unwrap().cmp(&right.read().unwrap())
            }
            (Value::Immutable(inner), Value::Mutable(inner_locked))
            | (Value::Mutable(inner_locked), Value::Immutable(inner)) => {
                inner_locked.read().unwrap().cmp(inner)
            }
        }
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Immutable(inner) => inner.serialize(serializer),
            Value::Mutable(inner_locked) => inner_locked.read().unwrap().serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        ValueData::deserialize(deserializer).map(|data| Value::Immutable(Arc::new(data)))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueData {
    Boolean(bool),
    Float(f64),
    Function(Function),
    Integer(i64),
    List(Vec<Value>),
    Map(BTreeMap<Identifier, Value>),
    Range(Range<i64>),
    String(String),
    Struct(Struct),
}

impl ValueData {
    fn r#type(&self) -> Type {
        match self {
            ValueData::Boolean(_) => Type::Boolean,
            ValueData::Float(_) => Type::Float,
            ValueData::Function(function) => Type::Function {
                type_parameters: function.type_parameters.clone(),
                value_parameters: function.value_parameters.clone(),
                return_type: function.return_type.as_ref().cloned().map(Box::new),
            },
            ValueData::Integer(_) => Type::Integer,
            ValueData::List(values) => {
                let item_type = values.first().unwrap().r#type();

                Type::List {
                    item_type: Box::new(item_type),
                    length: values.len(),
                }
            }
            ValueData::Map(value_map) => {
                let mut type_map = BTreeMap::new();

                for (identifier, value) in value_map {
                    let r#type = value.r#type();

                    type_map.insert(identifier.clone(), r#type);
                }

                Type::Map(type_map)
            }
            ValueData::Range(_) => Type::Range,
            ValueData::String(_) => Type::String,
            ValueData::Struct(r#struct) => match r#struct {
                Struct::Unit { name } => Type::Struct(StructType::Unit { name: name.clone() }),
                Struct::Tuple { .. } => todo!(),
                Struct::Fields { .. } => todo!(),
            },
        }
    }

    fn get_property(&self, property: &Identifier) -> Option<Value> {
        match self {
            ValueData::List(list) => {
                if property.as_str() == "length" {
                    Some(Value::integer(list.len() as i64))
                } else {
                    None
                }
            }
            ValueData::Map(value_map) => value_map.get(property).cloned(),
            _ => None,
        }
    }
}

impl Display for ValueData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ValueData::Boolean(boolean) => write!(f, "{boolean}"),
            ValueData::Float(float) => {
                if float == &f64::INFINITY {
                    return write!(f, "Infinity");
                }

                if float == &f64::NEG_INFINITY {
                    return write!(f, "-Infinity");
                }

                write!(f, "{float}")?;

                if &float.floor() == float {
                    write!(f, ".0")?;
                }

                Ok(())
            }
            ValueData::Function(function) => write!(f, "{function}"),
            ValueData::Integer(integer) => write!(f, "{integer}"),
            ValueData::List(list) => {
                write!(f, "[")?;

                for (index, value) in list.iter().enumerate() {
                    if index == list.len() - 1 {
                        write!(f, "{}", value)?;
                    } else {
                        write!(f, "{}, ", value)?;
                    }
                }

                write!(f, "]")
            }
            ValueData::Map(map) => {
                write!(f, "{{ ")?;

                for (index, (key, value)) in map.iter().enumerate() {
                    write!(f, "{key} = {value}")?;

                    if index != map.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, " }}")
            }
            ValueData::Range(range) => write!(f, "{}..{}", range.start, range.end),
            ValueData::String(string) => write!(f, "{string}"),
            ValueData::Struct(r#struct) => write!(f, "{struct}"),
        }
    }
}

impl Eq for ValueData {}

impl PartialOrd for ValueData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValueData {
    fn cmp(&self, other: &Self) -> Ordering {
        use ValueData::*;

        match (self, other) {
            (Boolean(left), Boolean(right)) => left.cmp(right),
            (Boolean(_), _) => Ordering::Greater,
            (Float(left), Float(right)) => left.total_cmp(right),
            (Float(_), _) => Ordering::Greater,
            (Function(left), Function(right)) => left.cmp(right),
            (Function(_), _) => Ordering::Greater,
            (Integer(left), Integer(right)) => left.cmp(right),
            (Integer(_), _) => Ordering::Greater,
            (List(left), List(right)) => left.cmp(right),
            (List(_), _) => Ordering::Greater,
            (Map(left), Map(right)) => left.cmp(right),
            (Map(_), _) => Ordering::Greater,
            (Range(left), Range(right)) => {
                let start_cmp = left.start.cmp(&right.start);

                if start_cmp.is_eq() {
                    left.end.cmp(&right.end)
                } else {
                    start_cmp
                }
            }
            (Range(_), _) => Ordering::Greater,
            (String(left), String(right)) => left.cmp(right),
            (String(_), _) => Ordering::Greater,
            (Struct(left), Struct(right)) => left.cmp(right),
            (Struct(_), _) => Ordering::Greater,
        }
    }
}

impl Serialize for ValueData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ValueData::Boolean(boolean) => serializer.serialize_bool(*boolean),
            ValueData::Float(float) => serializer.serialize_f64(*float),
            ValueData::Function(function) => function.serialize(serializer),
            ValueData::Integer(integer) => serializer.serialize_i64(*integer),
            ValueData::List(list) => {
                let mut list_ser = serializer.serialize_seq(Some(list.len()))?;

                for item in list {
                    list_ser.serialize_element(&item)?;
                }

                list_ser.end()
            }
            ValueData::Map(map) => {
                let mut map_ser = serializer.serialize_map(Some(map.len()))?;

                for (identifier, value) in map {
                    map_ser.serialize_entry(identifier, value)?;
                }

                map_ser.end()
            }
            ValueData::Range(range) => {
                let mut tuple_ser = serializer.serialize_tuple(2)?;

                tuple_ser.serialize_element(&range.start)?;
                tuple_ser.serialize_element(&range.end)?;

                tuple_ser.end()
            }
            ValueData::String(string) => serializer.serialize_str(string),
            ValueData::Struct(r#struct) => r#struct.serialize(serializer),
        }
    }
}

struct ValueInnerVisitor;

impl<'de> Visitor<'de> for ValueInnerVisitor {
    type Value = ValueData;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .write_str("a boolean, float, function, integer, list, map, range, string or structure")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ValueData::Boolean(v))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ValueData::Integer(v))
    }

    fn visit_i128<E>(self, _: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        todo!()
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u64(v as u64)
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u64(v as u64)
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u64(v as u64)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ValueData::Integer(v as i64))
    }

    fn visit_u128<E>(self, _: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        todo!()
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_f64(v as f64)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ValueData::Float(v))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v.encode_utf8(&mut [0u8; 4]))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ValueData::String(v.to_string()))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&v)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Bytes(v),
            &self,
        ))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_bytes(v)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_bytes(&v)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Option,
            &self,
        ))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let _ = deserializer;
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Option,
            &self,
        ))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Unit,
            &self,
        ))
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let _ = deserializer;
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::NewtypeStruct,
            &self,
        ))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut list = Vec::with_capacity(seq.size_hint().unwrap_or(10));

        while let Some(element) = seq.next_element()? {
            list.push(element);
        }

        Ok(ValueData::List(list))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut btree = BTreeMap::new();

        while let Some((key, value)) = map.next_entry()? {
            btree.insert(key, value);
        }

        Ok(ValueData::Map(btree))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        let _ = data;
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Enum,
            &self,
        ))
    }
}

impl<'de> Deserialize<'de> for ValueData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueInnerVisitor)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Function {
    pub name: Identifier,
    pub type_parameters: Option<Vec<Type>>,
    pub value_parameters: Option<Vec<(Identifier, Type)>>,
    pub return_type: Option<Type>,
    pub body: AbstractSyntaxTree,
}

impl Function {
    pub fn call(
        self,
        _type_arguments: Option<Vec<Type>>,
        value_arguments: Option<Vec<Value>>,
        context: &Context,
    ) -> Result<Option<Value>, VmError> {
        let new_context = Context::with_variables_from(context);

        if let (Some(value_parameters), Some(value_arguments)) =
            (self.value_parameters, value_arguments)
        {
            for ((identifier, _), value) in value_parameters.into_iter().zip(value_arguments) {
                new_context.set_value(identifier, value);
            }
        }

        let mut vm = Vm::new(self.body, new_context);

        vm.run()
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;

        if let Some(type_parameters) = &self.type_parameters {
            write!(f, "<")?;

            for (index, type_parameter) in type_parameters.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }

                write!(f, "{}", type_parameter)?;
            }

            write!(f, ">")?;
        }

        write!(f, "(")?;

        if let Some(value_paramers) = &self.value_parameters {
            for (index, (identifier, r#type)) in value_paramers.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }

                write!(f, "{identifier}: {type}")?;
            }
        }

        write!(f, ") {{")?;

        for statement in &self.body.statements {
            write!(f, "{}", statement)?;
        }

        write!(f, "}}")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Struct {
    Unit {
        name: Identifier,
    },
    Tuple {
        name: Identifier,
        fields: Vec<Value>,
    },
    Fields {
        name: Identifier,
        fields: Vec<(Identifier, Value)>,
    },
}

impl Struct {
    pub fn name(&self) -> &Identifier {
        match self {
            Struct::Unit { name } => name,
            Struct::Tuple { name, .. } => name,
            Struct::Fields { name, .. } => name,
        }
    }
}

impl Display for Struct {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Struct::Unit { name } => write!(f, "{}", name),
            Struct::Tuple { name, fields } => {
                write!(f, "{}(", name)?;

                for (index, field) in fields.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}", field)?;
                }

                write!(f, ")")
            }
            Struct::Fields { name, fields } => {
                write!(f, "{} {{", name)?;

                for (index, (identifier, r#type)) in fields.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}: {}", identifier, r#type)?;
                }

                write!(f, "}}")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValueError {
    CannotAdd(Value, Value),
    CannotAnd(Value, Value),
    CannotDivide(Value, Value),
    CannotGreaterThan(Value, Value),
    CannotGreaterThanOrEqual(Value, Value),
    CannotLessThan(Value, Value),
    CannotLessThanOrEqual(Value, Value),
    CannotModulo(Value, Value),
    CannotMultiply(Value, Value),
    CannotMutate(Value),
    CannotSubtract(Value, Value),
    CannotOr(Value, Value),
    DivisionByZero,
    ExpectedList(Value),
    IndexOutOfBounds { value: Value, index: i64 },
}

impl Error for ValueError {}

impl Display for ValueError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ValueError::CannotAdd(left, right) => write!(f, "Cannot add {} and {}", left, right),
            ValueError::CannotAnd(left, right) => write!(
                f,
                "Cannot use logical and operation on {} and {}",
                left, right
            ),
            ValueError::CannotDivide(left, right) => {
                write!(f, "Cannot divide {} by {}", left, right)
            }
            ValueError::CannotModulo(left, right) => {
                write!(f, "Cannot modulo {} by {}", left, right)
            }
            ValueError::CannotMultiply(left, right) => {
                write!(f, "Cannot multiply {} and {}", left, right)
            }
            ValueError::CannotMutate(value) => write!(f, "Cannot mutate {}", value),
            ValueError::CannotSubtract(left, right) => {
                write!(f, "Cannot subtract {} and {}", left, right)
            }
            ValueError::CannotLessThan(left, right)
            | ValueError::CannotLessThanOrEqual(left, right)
            | ValueError::CannotGreaterThan(left, right)
            | ValueError::CannotGreaterThanOrEqual(left, right) => {
                write!(f, "Cannot compare {} and {}", left, right)
            }
            ValueError::CannotOr(left, right) => {
                write!(
                    f,
                    "Cannot use logical or operation on {} and {}",
                    left, right
                )
            }
            ValueError::DivisionByZero => write!(f, "Division by zero"),
            ValueError::IndexOutOfBounds { value, index } => {
                write!(f, "{} does not have an index of {}", value, index)
            }
            ValueError::ExpectedList(value) => write!(f, "{} is not a list", value),
        }
    }
}
