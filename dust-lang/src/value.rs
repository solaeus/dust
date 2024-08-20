//! Dust value representation
use std::{
    cmp::Ordering,
    collections::HashMap,
    error::Error,
    fmt::{self, Display, Formatter},
    ops::{Index, Range, RangeInclusive},
    rc::Weak,
    sync::{Arc, RwLock},
};

use serde::{
    de::{self, MapAccess, Visitor},
    ser::{SerializeMap, SerializeStruct, SerializeStructVariant},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{
    AbstractSyntaxTree, BuiltInFunction, Context, EnumType, FunctionType, Identifier,
    RangeableType, RuntimeError, StructType, Type, Vm,
};

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
    Boolean(bool),
    Byte(u8),
    Character(char),
    Enum(Enum),
    Float(f64),
    Function(Function),
    Integer(i64),
    List(Vec<Value>),
    Map(HashMap<Identifier, Value>),
    Mutable(Arc<RwLock<Value>>),
    Range(Range<Rangeable>),
    RangeInclusive(RangeInclusive<Rangeable>),
    String(String),
    Struct(Struct),
    Tuple(Vec<Value>),
}

impl Value {
    pub fn map<T: Into<HashMap<Identifier, Value>>>(pairs: T) -> Value {
        Value::Map(pairs.into())
    }

    pub fn mutable(value: Value) -> Value {
        Value::Mutable(Arc::new(RwLock::new(value)))
    }

    pub fn mutable_from<T: Into<Value>>(into_value: T) -> Value {
        Value::Mutable(Arc::new(RwLock::new(into_value.into())))
    }

    pub fn byte_range(start: u8, end: u8) -> Value {
        Value::Range(Rangeable::Byte(start)..Rangeable::Byte(end))
    }

    pub fn byte_range_inclusive(start: u8, end: u8) -> Value {
        Value::RangeInclusive(Rangeable::Byte(start)..=Rangeable::Byte(end))
    }

    pub fn character_range(start: char, end: char) -> Value {
        Value::Range(Rangeable::Character(start)..Rangeable::Character(end))
    }

    pub fn character_range_inclusive(start: char, end: char) -> Value {
        Value::RangeInclusive(Rangeable::Character(start)..=Rangeable::Character(end))
    }

    pub fn float_range(start: f64, end: f64) -> Value {
        Value::Range(Rangeable::Float(start)..Rangeable::Float(end))
    }

    pub fn float_range_inclusive(start: f64, end: f64) -> Value {
        Value::RangeInclusive(Rangeable::Float(start)..=Rangeable::Float(end))
    }

    pub fn integer_range(start: i64, end: i64) -> Value {
        Value::Range(Rangeable::Integer(start)..Rangeable::Integer(end))
    }

    pub fn integer_range_inclusive(start: i64, end: i64) -> Value {
        Value::RangeInclusive(Rangeable::Integer(start)..=Rangeable::Integer(end))
    }

    pub fn string<T: ToString>(to_string: T) -> Value {
        Value::String(to_string.to_string())
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(value) => Some(*value),
            Value::Mutable(locked) => locked.read().unwrap().as_boolean(),
            _ => None,
        }
    }

    pub fn as_byte(&self) -> Option<u8> {
        match self {
            Value::Byte(value) => Some(*value),
            Value::Mutable(locked) => locked.read().unwrap().as_byte(),
            _ => None,
        }
    }

    pub fn as_character(&self) -> Option<char> {
        match self {
            Value::Character(value) => Some(*value),
            Value::Mutable(locked) => locked.read().unwrap().as_character(),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(value) => Some(*value),
            Value::Mutable(locked) => locked.read().unwrap().as_float(),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(value) => Some(*value),
            Value::Mutable(locked) => locked.read().unwrap().as_integer(),
            _ => None,
        }
    }

    pub fn as_mutable(&self) -> Result<&Arc<RwLock<Value>>, ValueError> {
        match self {
            Value::Mutable(inner) => Ok(inner),
            _ => Err(ValueError::CannotMutate(self.clone())),
        }
    }

    pub fn into_mutable(self) -> Value {
        match self {
            Value::Mutable(_) => self,
            immutable => Value::Mutable(Arc::new(RwLock::new(immutable))),
        }
    }

    pub fn is_mutable(&self) -> bool {
        matches!(self, Value::Mutable(_))
    }

    pub fn mutate(&self, other: Value) -> Result<(), ValueError> {
        match self {
            Value::Mutable(inner) => *inner.write().unwrap() = other,
            _ => return Err(ValueError::CannotMutate(self.clone())),
        };

        Ok(())
    }

    pub fn r#type(&self) -> Type {
        match self {
            Value::Boolean(_) => Type::Boolean,
            Value::Byte(_) => Type::Byte,
            Value::Character(_) => Type::Character,
            Value::Enum(Enum { r#type, .. }) => Type::Enum(r#type.clone()),
            Value::Float(_) => Type::Float,
            Value::Function(Function::BuiltIn(built_in_function)) => Type::Function(FunctionType {
                name: Identifier::new(built_in_function.name()),
                type_parameters: built_in_function.type_parameters(),
                value_parameters: built_in_function.value_parameters(),
                return_type: built_in_function.return_type().map(Box::new),
            }),
            Value::Function(Function::Parsed { name, r#type, body }) => {
                Type::Function(r#type.clone())
            }
            Value::Integer(_) => Type::Integer,
            Value::List(values) => {
                let item_type = values.first().unwrap().r#type();

                Type::List {
                    item_type: Box::new(item_type),
                    length: values.len(),
                }
            }
            Value::Map(map) => {
                let pairs = map
                    .iter()
                    .map(|(key, value)| (key.clone(), value.r#type()))
                    .collect();

                Type::Map { pairs }
            }
            Value::Mutable(locked) => locked.read().unwrap().r#type(),
            Value::Range(range) => Type::Range {
                r#type: range.start.r#type(),
            },
            Value::RangeInclusive(range_inclusive) => {
                let rangeable_type = range_inclusive.start().r#type();

                Type::Range {
                    r#type: rangeable_type,
                }
            }
            Value::String(_) => Type::String,
            Value::Struct(r#struct) => match r#struct {
                Struct::Unit { name } => Type::Struct(StructType::Unit { name: name.clone() }),
                Struct::Tuple { name, fields } => {
                    let types = fields.iter().map(|field| field.r#type()).collect();

                    Type::Struct(StructType::Tuple {
                        name: name.clone(),
                        fields: types,
                    })
                }
                Struct::Fields { name, fields } => {
                    let types = fields
                        .iter()
                        .map(|(identifier, value)| (identifier.clone(), value.r#type()))
                        .collect();

                    Type::Struct(StructType::Fields {
                        name: name.clone(),
                        fields: types,
                    })
                }
            },
            Value::Tuple(values) => {
                let fields = values.iter().map(|value| value.r#type()).collect();

                Type::Tuple(fields)
            }
        }
    }

    pub fn get_field(&self, field: &Identifier) -> Option<Value> {
        if let "to_string" = field.as_str() {
            return Some(Value::Function(Function::BuiltIn(
                BuiltInFunction::ToString {
                    argument: Box::new(self.clone()),
                },
            )));
        }

        match self {
            Value::Mutable(inner) => inner.read().unwrap().get_field(field),
            Value::Struct(Struct::Fields { fields, .. }) => fields.get(field).cloned(),
            Value::Map(pairs) => pairs.get(field).cloned(),
            _ => None,
        }
    }

    pub fn get_index(&self, index: Value) -> Result<Option<Value>, ValueError> {
        match (self, index) {
            (Value::Mutable(left), Value::Mutable(right)) => {
                return left
                    .read()
                    .unwrap()
                    .get_index(right.read().unwrap().clone());
            }
            (Value::Mutable(locked), index) => {
                return locked.read().unwrap().get_index(index);
            }
            (left, Value::Mutable(locked)) => {
                return left.get_index(locked.read().unwrap().clone());
            }
            (Value::List(values), Value::Integer(integer)) => {
                let index = integer as usize;

                return Ok(values.get(index).cloned());
            }
            (Value::List(values), Value::Range(range)) => match (range.start, range.end) {
                (Rangeable::Integer(start), Rangeable::Integer(end)) => {
                    let start = start as usize;
                    let end = end as usize;

                    return Ok(values
                        .get(start..end)
                        .map(|values| Value::List(values.to_vec())));
                }
                (start, end) => Err(ValueError::CannotIndex {
                    value: self.clone(),
                    index: Value::Range(start..end),
                }),
            },
            (Value::String(string), Value::Range(range)) => match (range.start, range.end) {
                (Rangeable::Integer(start), Rangeable::Integer(end)) => {
                    let start = start as usize;
                    let end = end as usize;

                    return Ok(string.get(start..end).map(Value::string));
                }
                (start, end) => Err(ValueError::CannotIndex {
                    value: self.clone(),
                    index: Value::Range(start..end),
                }),
            },
            (Value::Range(range), Value::Integer(index)) => match (range.start, range.end) {
                (Rangeable::Integer(start), Rangeable::Integer(end)) => {
                    Ok((start..end).nth(index as usize).map(Value::Integer))
                }
                (start, end) => Err(ValueError::CannotIndex {
                    value: self.clone(),
                    index: Value::Range(start..end),
                }),
            },
            (Value::String(string), Value::Integer(integer)) => {
                let index = integer as usize;

                return Ok(string.chars().nth(index).map(Value::Character));
            }
            (value, index) => Err(ValueError::CannotIndex {
                value: value.clone(),
                index,
            }),
        }
    }

    pub fn add(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left + right)),
            (Value::Integer(left), Value::Integer(right)) => {
                Ok(Value::Integer(left.saturating_add(*right)))
            }
            (Value::String(left), Value::String(right)) => {
                Ok(Value::String(format!("{}{}", left, right)))
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                let left = left.read().unwrap();
                let right = right.read().unwrap();

                left.add(&right)
            }
            _ => Err(ValueError::CannotAdd(self.clone(), other.clone())),
        }
    }

    pub fn add_assign(&self, other: &Value) -> Result<(), ValueError> {
        match (self, other) {
            (Value::Mutable(left), Value::Mutable(right)) => {
                match (&mut *left.write().unwrap(), &*right.read().unwrap()) {
                    (Value::Float(left), Value::Float(right)) => {
                        *left += right;
                        return Ok(());
                    }
                    (Value::Integer(left), Value::Integer(right)) => {
                        *left = left.saturating_add(*right);
                        return Ok(());
                    }
                    (Value::String(left), Value::String(right)) => {
                        (*left).push_str(right);
                        return Ok(());
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), right) => match (&mut *left.write().unwrap(), right) {
                (Value::Float(left), Value::Float(right)) => {
                    *left += right;
                    return Ok(());
                }
                (Value::Integer(left), Value::Integer(right)) => {
                    *left = left.saturating_add(*right);
                    return Ok(());
                }
                (Value::String(left), Value::String(right)) => {
                    left.push_str(right);
                    return Ok(());
                }
                _ => {}
            },
            _ => {}
        }

        Err(ValueError::CannotMutate(self.clone()))
    }

    pub fn subtract(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left - right)),
            (Value::Integer(left), Value::Integer(right)) => {
                Ok(Value::Integer(left.saturating_sub(*right)))
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                let left = left.read().unwrap();
                let right = right.read().unwrap();

                left.subtract(&right)
            }
            _ => Err(ValueError::CannotSubtract(self.clone(), other.clone())),
        }
    }

    pub fn subtract_assign(&self, other: &Value) -> Result<(), ValueError> {
        match (self, other) {
            (Value::Mutable(left), Value::Mutable(right)) => {
                match (&mut *left.write().unwrap(), &*right.read().unwrap()) {
                    (Value::Float(left), Value::Float(right)) => {
                        *left -= right;
                        return Ok(());
                    }
                    (Value::Integer(left), Value::Integer(right)) => {
                        *left = left.saturating_sub(*right);
                        return Ok(());
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), right) => match (&mut *left.write().unwrap(), right) {
                (Value::Float(left), Value::Float(right)) => {
                    *left -= right;
                    return Ok(());
                }
                (Value::Integer(left), Value::Integer(right)) => {
                    *left = left.saturating_sub(*right);
                    return Ok(());
                }
                _ => {}
            },
            _ => {}
        }

        Err(ValueError::CannotSubtract(self.clone(), other.clone()))
    }

    pub fn multiply(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left * right)),
            (Value::Integer(left), Value::Integer(right)) => {
                Ok(Value::Integer(left.saturating_mul(*right)))
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                let left = left.read().unwrap();
                let right = right.read().unwrap();

                left.multiply(&right)
            }
            _ => Err(ValueError::CannotMultiply(self.clone(), other.clone())),
        }
    }

    pub fn multiply_assign(&self, other: &Value) -> Result<(), ValueError> {
        match (self, other) {
            (Value::Mutable(left), Value::Mutable(right)) => {
                match (&mut *left.write().unwrap(), &*right.read().unwrap()) {
                    (Value::Float(left), Value::Float(right)) => {
                        *left *= right;
                        return Ok(());
                    }
                    (Value::Integer(left), Value::Integer(right)) => {
                        *left = left.saturating_mul(*right);
                        return Ok(());
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), right) => match (&mut *left.write().unwrap(), right) {
                (Value::Float(left), Value::Float(right)) => {
                    *left *= right;
                    return Ok(());
                }
                (Value::Integer(left), Value::Integer(right)) => {
                    *left = left.saturating_mul(*right);
                    return Ok(());
                }
                _ => {}
            },
            _ => {}
        }

        Err(ValueError::CannotMultiply(self.clone(), other.clone()))
    }

    pub fn divide(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left / right)),
            (Value::Integer(left), Value::Integer(right)) => {
                Ok(Value::Integer(left.saturating_div(*right)))
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                let left = left.read().unwrap();
                let right = right.read().unwrap();

                left.divide(&right)
            }
            _ => Err(ValueError::CannotDivide(self.clone(), other.clone())),
        }
    }

    pub fn divide_assign(&self, other: &Value) -> Result<(), ValueError> {
        match (self, other) {
            (Value::Mutable(left), Value::Mutable(right)) => {
                match (&mut *left.write().unwrap(), &*right.read().unwrap()) {
                    (Value::Float(left), Value::Float(right)) => {
                        *left /= right;
                        return Ok(());
                    }
                    (Value::Integer(left), Value::Integer(right)) => {
                        *left = (*left as f64 / *right as f64) as i64;
                        return Ok(());
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), right) => match (&mut *left.write().unwrap(), right) {
                (Value::Float(left), Value::Float(right)) => {
                    *left /= right;
                    return Ok(());
                }
                (Value::Integer(left), Value::Integer(right)) => {
                    *left = (*left as f64 / *right as f64) as i64;
                    return Ok(());
                }
                _ => {}
            },
            _ => {}
        }

        Err(ValueError::CannotDivide(self.clone(), other.clone()))
    }

    pub fn modulo(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left % right)),
            (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(left % right)),
            (Value::Mutable(left), Value::Mutable(right)) => {
                let left = left.read().unwrap();
                let right = right.read().unwrap();

                left.modulo(&right)
            }
            _ => Err(ValueError::CannotModulo(self.clone(), other.clone())),
        }
    }

    pub fn modulo_assign(&self, other: &Value) -> Result<(), ValueError> {
        match (self, other) {
            (Value::Mutable(left), Value::Mutable(right)) => {
                match (&mut *left.write().unwrap(), &*right.read().unwrap()) {
                    (Value::Float(left), Value::Float(right)) => {
                        *left %= right;
                        return Ok(());
                    }
                    (Value::Integer(left), Value::Integer(right)) => {
                        *left %= right;
                        return Ok(());
                    }
                    _ => {}
                }
            }
            (Value::Mutable(left), right) => match (&mut *left.write().unwrap(), right) {
                (Value::Float(left), Value::Float(right)) => {
                    *left %= right;
                    return Ok(());
                }
                (Value::Integer(left), Value::Integer(right)) => {
                    *left %= right;
                    return Ok(());
                }
                _ => {}
            },
            _ => {}
        }

        Err(ValueError::CannotModulo(self.clone(), other.clone()))
    }

    pub fn equal(&self, other: &Value) -> Value {
        let is_equal = match (self, other) {
            (Value::Boolean(left), Value::Boolean(right)) => left == right,
            (Value::Byte(left), Value::Byte(right)) => left == right,
            (Value::Character(left), Value::Character(right)) => left == right,
            (Value::Float(left), Value::Float(right)) => left == right,
            (Value::Function(left), Value::Function(right)) => left == right,
            (Value::Integer(left), Value::Integer(right)) => left == right,
            (Value::List(left), Value::List(right)) => {
                if left.len() != right.len() {
                    return Value::Boolean(false);
                }

                for (left, right) in left.iter().zip(right.iter()) {
                    if let Value::Boolean(false) = left.equal(right) {
                        return Value::Boolean(false);
                    }
                }

                true
            }
            (Value::Range(left), Value::Range(right)) => {
                left.start == right.start && left.end == right.end
            }
            (Value::RangeInclusive(left), Value::RangeInclusive(right)) => {
                left.start() == right.start() && left.end() == right.end()
            }
            (Value::String(left), Value::String(right)) => left == right,
            (Value::Struct(left), Value::Struct(right)) => left == right,
            (Value::Mutable(left), Value::Mutable(right)) => {
                let left = left.read().unwrap();
                let right = right.read().unwrap();

                return left.equal(&right);
            }
            (Value::Mutable(locked), immutable) | (immutable, Value::Mutable(locked)) => {
                let locked = locked.read().unwrap();

                return locked.equal(immutable);
            }
            _ => false,
        };

        Value::Boolean(is_equal)
    }

    pub fn not_equal(&self, other: &Value) -> Value {
        if let Value::Boolean(is_equal) = self.equal(other) {
            Value::Boolean(!is_equal)
        } else {
            Value::Boolean(true)
        }
    }

    pub fn less_than(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Float(left), Value::Float(right)) => Ok(Value::Boolean(left < right)),
            (Value::Integer(left), Value::Integer(right)) => Ok(Value::Boolean(left < right)),
            (Value::Float(left), Value::Integer(right)) => {
                Ok(Value::Boolean(*left < *right as f64))
            }
            (Value::Integer(left), Value::Float(right)) => {
                Ok(Value::Boolean((*left as f64) < *right))
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                let left = left.read().unwrap();
                let right = right.read().unwrap();

                left.less_than(&right)
            }
            (Value::Mutable(left), right) => {
                let left = left.read().unwrap();

                left.less_than(right)
            }
            (left, Value::Mutable(right)) => {
                let right = right.read().unwrap();

                left.less_than(&right)
            }
            _ => Err(ValueError::CannotLessThan(self.clone(), other.clone())),
        }
    }

    pub fn less_than_or_equal(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Float(left), Value::Float(right)) => Ok(Value::Boolean(left <= right)),
            (Value::Integer(left), Value::Integer(right)) => Ok(Value::Boolean(left <= right)),
            (Value::Float(left), Value::Integer(right)) => {
                Ok(Value::Boolean(*left <= *right as f64))
            }
            (Value::Integer(left), Value::Float(right)) => {
                Ok(Value::Boolean(*left as f64 <= *right))
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                let left = left.read().unwrap();
                let right = right.read().unwrap();

                left.less_than_or_equal(&right)
            }
            (Value::Mutable(left), right) => {
                let left = left.read().unwrap();

                left.less_than_or_equal(right)
            }
            (left, Value::Mutable(right)) => {
                let right = right.read().unwrap();

                left.less_than_or_equal(&right)
            }
            _ => Err(ValueError::CannotLessThanOrEqual(
                self.clone(),
                other.clone(),
            )),
        }
    }

    pub fn greater_than(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Float(left), Value::Float(right)) => Ok(Value::Boolean(left > right)),
            (Value::Integer(left), Value::Integer(right)) => Ok(Value::Boolean(left > right)),
            (Value::Float(left), Value::Integer(right)) => {
                Ok(Value::Boolean(*left > *right as f64))
            }
            (Value::Integer(left), Value::Float(right)) => {
                Ok(Value::Boolean(*left as f64 > *right))
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                let left = left.read().unwrap();
                let right = right.read().unwrap();

                left.greater_than(&right)
            }
            (Value::Mutable(left), right) => {
                let left = left.read().unwrap();

                left.greater_than(right)
            }
            (left, Value::Mutable(right)) => {
                let right = right.read().unwrap();

                left.greater_than(&right)
            }
            _ => Err(ValueError::CannotGreaterThan(self.clone(), other.clone())),
        }
    }

    pub fn greater_than_or_equal(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Float(left), Value::Float(right)) => Ok(Value::Boolean(left >= right)),
            (Value::Integer(left), Value::Integer(right)) => Ok(Value::Boolean(left >= right)),
            (Value::Float(left), Value::Integer(right)) => {
                Ok(Value::Boolean(*left >= *right as f64))
            }
            (Value::Integer(left), Value::Float(right)) => {
                Ok(Value::Boolean(*left as f64 >= *right))
            }
            (Value::Mutable(left), Value::Mutable(right)) => {
                let left = left.read().unwrap();
                let right = right.read().unwrap();

                left.greater_than_or_equal(&right)
            }
            (Value::Mutable(left), right) => {
                let left = left.read().unwrap();

                left.greater_than_or_equal(right)
            }
            (left, Value::Mutable(right)) => {
                let right = right.read().unwrap();

                left.greater_than_or_equal(&right)
            }
            _ => Err(ValueError::CannotGreaterThanOrEqual(
                self.clone(),
                other.clone(),
            )),
        }
    }

    pub fn and(&self, other: &Value) -> Result<Value, ValueError> {
        match (self.as_boolean(), other.as_boolean()) {
            (Some(left), Some(right)) => Ok(Value::Boolean(left && right)),
            _ => Err(ValueError::CannotAnd(self.clone(), other.clone())),
        }
    }

    pub fn or(&self, other: &Value) -> Result<Value, ValueError> {
        match (self.as_boolean(), other.as_boolean()) {
            (Some(left), Some(right)) => Ok(Value::Boolean(left || right)),
            _ => Err(ValueError::CannotOr(self.clone(), other.clone())),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value::Byte(value)
    }
}

impl From<char> for Value {
    fn from(value: char) -> Self {
        Value::Character(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Integer(value as i64)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Mutable(inner_locked) => {
                let inner = inner_locked.read().unwrap();

                write!(f, "{inner}")
            }
            Value::Boolean(boolean) => write!(f, "{boolean}"),
            Value::Byte(byte) => write!(f, "{byte}"),
            Value::Character(character) => write!(f, "{character}"),
            Value::Enum(r#enum) => write!(f, "{enum}"),
            Value::Float(float) => write!(f, "{float}"),
            Value::Function(function) => write!(f, "{function}"),
            Value::Integer(integer) => write!(f, "{integer}"),
            Value::Map(pairs) => {
                write!(f, "{{ ")?;

                for (index, (key, value)) in pairs.iter().enumerate() {
                    write!(f, "{key}: {value}")?;

                    if index < pairs.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, " }}")
            }
            Value::List(list) => {
                write!(f, "[")?;

                for (index, value) in list.iter().enumerate() {
                    write!(f, "{}", value)?;

                    if index < list.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, "]")
            }
            Value::Range(Range { start, end }) => {
                write!(f, "{start}..{end}")
            }
            Value::RangeInclusive(inclusive) => {
                let start = inclusive.start();
                let end = inclusive.end();

                write!(f, "{start}..={end}")
            }
            Value::String(string) => write!(f, "{string}"),
            Value::Struct(r#struct) => write!(f, "{struct}"),
            Value::Tuple(fields) => {
                write!(f, "(")?;

                for (index, field) in fields.iter().enumerate() {
                    write!(f, "{}", field)?;

                    if index < fields.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ")")
            }
        }
    }
}

impl Eq for Value {}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Boolean(left), Value::Boolean(right)) => left == right,
            (Value::Byte(left), Value::Byte(right)) => left == right,
            (Value::Character(left), Value::Character(right)) => left == right,
            (Value::Float(left), Value::Float(right)) => left == right,
            (Value::Function(left), Value::Function(right)) => left == right,
            (Value::Integer(left), Value::Integer(right)) => left == right,
            (Value::List(left), Value::List(right)) => left == right,
            (Value::Map(left), Value::Map(right)) => left == right,
            (Value::Mutable(left), Value::Mutable(right)) => {
                let left = &*left.read().unwrap();
                let right = &*right.read().unwrap();

                left == right
            }
            (Value::Range(left), Value::Range(right)) => left == right,
            (Value::RangeInclusive(left), Value::RangeInclusive(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::Struct(left), Value::Struct(right)) => left == right,
            (Value::Tuple(left), Value::Tuple(right)) => left == right,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Value::Boolean(left), Value::Boolean(right)) => left.cmp(right),
            (Value::Boolean(_), _) => Ordering::Greater,
            (Value::Byte(left), Value::Byte(right)) => left.cmp(right),
            (Value::Byte(_), _) => Ordering::Greater,
            (Value::Character(left), Value::Character(right)) => left.cmp(right),
            (Value::Character(_), _) => Ordering::Greater,
            (Value::Float(left), Value::Float(right)) => left.partial_cmp(right).unwrap(),
            (Value::Float(_), _) => Ordering::Greater,
            (Value::Function(left), Value::Function(right)) => left.cmp(right),
            (Value::Function(_), _) => Ordering::Greater,
            (Value::Integer(left), Value::Integer(right)) => left.cmp(right),
            (Value::Integer(_), _) => Ordering::Greater,
            (Value::List(left), Value::List(right)) => left.cmp(right),
            (Value::List(_), _) => Ordering::Greater,
            (Value::Mutable(left), Value::Mutable(right)) => {
                let left = left.read().unwrap();
                let right = right.read().unwrap();

                left.cmp(&right)
            }
            (Value::Mutable(_), _) => Ordering::Greater,
            (Value::Range(left), Value::Range(right)) => {
                let start_cmp = left.start.cmp(&right.start);

                if start_cmp.is_eq() {
                    left.end.cmp(&right.end)
                } else {
                    start_cmp
                }
            }
            (Value::Range(_), _) => Ordering::Greater,
            (Value::RangeInclusive(left), Value::RangeInclusive(right)) => {
                let start_cmp = left.start().cmp(right.start());

                if start_cmp.is_eq() {
                    left.end().cmp(right.end())
                } else {
                    start_cmp
                }
            }
            (Value::RangeInclusive(_), _) => Ordering::Greater,
            (Value::String(left), Value::String(right)) => left.cmp(right),
            (Value::String(_), _) => Ordering::Greater,
            (Value::Struct(left), Value::Struct(right)) => left.cmp(right),
            (Value::Struct(_), _) => Ordering::Greater,
            (Value::Tuple(left), Value::Tuple(right)) => left.cmp(right),
            _ => Ordering::Greater,
        }
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Mutable(inner_locked) => {
                let inner = inner_locked.read().unwrap();

                inner.serialize(serializer)
            }
            Value::Boolean(boolean) => serializer.serialize_bool(*boolean),
            Value::Byte(byte) => serializer.serialize_u8(*byte),
            Value::Character(character) => serializer.serialize_char(*character),
            Value::Enum(r#emum) => r#emum.serialize(serializer),
            Value::Float(float) => serializer.serialize_f64(*float),
            Value::Function(function) => function.serialize(serializer),
            Value::Integer(integer) => serializer.serialize_i64(*integer),
            Value::List(list) => list.serialize(serializer),
            Value::Map(pairs) => {
                let mut ser = serializer.serialize_map(Some(pairs.len()))?;

                for (key, value) in pairs {
                    ser.serialize_entry(key, value)?;
                }

                ser.end()
            }
            Value::Range(range) => range.serialize(serializer),
            Value::RangeInclusive(inclusive) => inclusive.serialize(serializer),
            Value::String(string) => serializer.serialize_str(string),
            Value::Struct(r#struct) => r#struct.serialize(serializer),
            Value::Tuple(tuple) => tuple.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
                Ok(Value::Boolean(value))
            }

            fn visit_u8<E>(self, value: u8) -> Result<Value, E> {
                Ok(Value::Byte(value))
            }

            fn visit_char<E>(self, value: char) -> Result<Value, E> {
                Ok(Value::Character(value))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
                Ok(Value::Float(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
                Ok(Value::Integer(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Value, E> {
                Ok(Value::String(value.to_string()))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Function {
    BuiltIn(BuiltInFunction),
    Parsed {
        name: Identifier,
        r#type: FunctionType,
        body: AbstractSyntaxTree,
    },
}

impl Function {
    pub fn call(
        self,
        _type_arguments: Option<Vec<Type>>,
        value_arguments: Option<Vec<Value>>,
        context: &Context,
    ) -> Result<Option<Value>, RuntimeError> {
        match self {
            Function::BuiltIn(built_in_function) => built_in_function
                .call(_type_arguments, value_arguments)
                .map_err(|error| RuntimeError::BuiltInFunctionError { error }),
            Function::Parsed { r#type, body, .. } => {
                let new_context = Context::with_data_from(context);

                if let (Some(value_parameters), Some(value_arguments)) =
                    (&r#type.value_parameters, value_arguments)
                {
                    for ((identifier, _), value) in value_parameters.iter().zip(value_arguments) {
                        new_context.set_variable_value(identifier.clone(), value);
                    }
                }

                let mut vm = Vm::new(body, new_context);

                vm.run()
            }
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Function::BuiltIn(built_in_function) => write!(f, "{}", built_in_function),
            Function::Parsed { name, r#type, body } => {
                write!(f, "fn {}", name)?;

                if let Some(type_parameters) = &r#type.type_parameters {
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

                if let Some(value_paramers) = &r#type.value_parameters {
                    for (index, (identifier, r#type)) in value_paramers.iter().enumerate() {
                        if index > 0 {
                            write!(f, ", ")?;
                        }

                        write!(f, "{identifier}: {type}")?;
                    }
                }

                write!(f, ") {{")?;

                for statement in &body.statements {
                    write!(f, "{}", statement)?;
                }

                write!(f, "}}")
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
        fields: HashMap<Identifier, Value>,
    },
}

impl PartialOrd for Struct {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Struct {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Struct::Unit { name: left }, Struct::Unit { name: right }) => left.cmp(right),
            (Struct::Unit { .. }, _) => Ordering::Greater,
            (
                Struct::Tuple {
                    name: left_name,
                    fields: left_fields,
                },
                Struct::Tuple {
                    name: right_name,
                    fields: right_fields,
                },
            ) => {
                let type_cmp = left_name.cmp(right_name);

                if type_cmp != Ordering::Equal {
                    return type_cmp;
                }

                left_fields.cmp(right_fields)
            }
            (Struct::Tuple { .. }, _) => Ordering::Greater,
            (
                Struct::Fields {
                    name: left_name,
                    fields: left_fields,
                },
                Struct::Fields {
                    name: right_name,
                    fields: right_fields,
                },
            ) => {
                let type_cmp = left_name.cmp(right_name);

                if type_cmp != Ordering::Equal {
                    return type_cmp;
                }

                left_fields.into_iter().cmp(right_fields.into_iter())
            }
            (Struct::Fields { .. }, _) => Ordering::Greater,
        }
    }
}

impl Display for Struct {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Struct::Unit { name } => write!(f, "{name}"),
            Struct::Tuple { name, fields } => {
                write!(f, "{name}(")?;

                for (index, field) in fields.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}", field)?;
                }

                write!(f, ")")
            }
            Struct::Fields { name, fields } => {
                write!(f, "{name} {{ ")?;

                for (index, (identifier, value)) in fields.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}: {}", identifier, value)?;
                }

                write!(f, " }}")
            }
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Rangeable {
    Byte(u8),
    Character(char),
    Float(f64),
    Integer(i64),
}

impl Rangeable {
    fn r#type(&self) -> RangeableType {
        match self {
            Rangeable::Byte(_) => RangeableType::Byte,
            Rangeable::Character(_) => RangeableType::Character,
            Rangeable::Float(_) => RangeableType::Float,
            Rangeable::Integer(_) => RangeableType::Integer,
        }
    }
}

impl Display for Rangeable {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Rangeable::Byte(byte) => write!(f, "{byte}"),
            Rangeable::Character(character) => write!(f, "{character}"),
            Rangeable::Float(float) => write!(f, "{float}"),
            Rangeable::Integer(integer) => write!(f, "{integer}"),
        }
    }
}

impl Eq for Rangeable {}

impl PartialOrd for Rangeable {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rangeable {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Rangeable::Byte(left), Rangeable::Byte(right)) => left.cmp(right),
            (Rangeable::Character(left), Rangeable::Character(right)) => left.cmp(right),
            (Rangeable::Float(left), Rangeable::Float(right)) => {
                left.to_bits().cmp(&right.to_bits())
            }
            (Rangeable::Integer(left), Rangeable::Integer(right)) => left.cmp(right),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Enum {
    pub r#type: EnumType,
    pub name: Identifier,
    pub variant_data: Struct,
}

impl Display for Enum {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Enum {
            name, variant_data, ..
        } = self;

        match &variant_data {
            Struct::Unit { name: variant_name } => write!(f, "{name}::{variant_name}"),
            Struct::Tuple {
                name: variant_name,
                fields,
            } => {
                write!(f, "{name}::{variant_name}(")?;

                for (index, field) in fields.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}", field)?;
                }

                write!(f, ")")
            }
            Struct::Fields {
                name: variant_name,
                fields,
            } => {
                write!(f, "{name}::{variant_name} {{ ")?;

                for (index, (identifier, value)) in fields.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}: {}", identifier, value)?;
                }

                write!(f, " }}")
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
    CannotIndex { value: Value, index: Value },
    CannotLessThan(Value, Value),
    CannotLessThanOrEqual(Value, Value),
    CannotMakeMutable,
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
            ValueError::CannotIndex { value, index } => {
                write!(f, "Cannot index {} with {}", value, index)
            }
            ValueError::CannotModulo(left, right) => {
                write!(f, "Cannot modulo {} by {}", left, right)
            }
            ValueError::CannotMultiply(left, right) => {
                write!(f, "Cannot multiply {} and {}", left, right)
            }
            ValueError::CannotMakeMutable => write!(
                f,
                "Failed to make mutable value because the value has an immutable reference to it"
            ),
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
