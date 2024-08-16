//! Dust value representation
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    error::Error,
    fmt::{self, Display, Formatter},
    ops::{Range, RangeInclusive},
    ptr,
    sync::{Arc, RwLock, RwLockWriteGuard},
};

use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{
    AbstractSyntaxTree, Context, EnumType, FunctionType, Identifier, StructType, Type, Vm, VmError,
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
    Enum { name: Identifier, r#type: EnumType },
    Float(f64),
    Function(Arc<Function>),
    Integer(i64),
    List(Vec<Value>),
    Mutable(Arc<RwLock<Value>>),
    Range(Range<Rangeable>),
    RangeInclusive(RangeInclusive<Rangeable>),
    String(String),
    Struct(Struct),
    Tuple(Vec<Value>),
}

impl Value {
    pub fn byte_range(start: u8, end: u8) -> Value {
        Value::Range(Rangeable::Byte(start)..Rangeable::Byte(end))
    }

    pub fn character_range(start: char, end: char) -> Value {
        Value::Range(Rangeable::Character(start)..Rangeable::Character(end))
    }

    pub fn float_range(start: f64, end: f64) -> Value {
        Value::Range(Rangeable::Float(start)..Rangeable::Float(end))
    }

    pub fn integer_range(start: i64, end: i64) -> Value {
        Value::Range(Rangeable::Integer(start)..Rangeable::Integer(end))
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

    pub fn as_mutable(&self) -> Result<&Arc<RwLock<Value>>, ValueError> {
        match self {
            Value::Mutable(inner) => Ok(inner),
            _ => Err(ValueError::CannotMutate(self.clone())),
        }
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
            Value::Enum { r#type, .. } => Type::Enum(r#type.clone()),
            Value::Float(_) => Type::Float,
            Value::Function(function) => Type::Function(function.r#type.clone()),
            Value::Integer(_) => Type::Integer,
            Value::List(values) => {
                let item_type = values.first().unwrap().r#type();

                Type::List {
                    item_type: Box::new(item_type),
                    length: values.len(),
                }
            }
            Value::Mutable(locked) => locked.read().unwrap().r#type(),
            Value::Range(_) => Type::Range,
            Value::RangeInclusive(_) => Type::Range,
            Value::String(_) => Type::String,
            Value::Struct(r#struct) => match r#struct {
                Struct::Unit { r#type } => r#type.clone(),
                Struct::Tuple { r#type, .. } => r#type.clone(),
                Struct::Fields { r#type, .. } => r#type.clone(),
            },
            Value::Tuple(values) => {
                let item_types = values.iter().map(Value::r#type).collect();

                Type::Tuple(item_types)
            }
        }
    }

    pub fn get_field(&self, field: &Identifier) -> Option<Value> {
        match self {
            Value::Struct(Struct::Fields { fields, .. }) => {
                fields.iter().find_map(|(identifier, value)| {
                    if identifier == field {
                        Some(value.clone())
                    } else {
                        None
                    }
                })
            }
            Value::Mutable(inner) => inner.clone().read().unwrap().get_field(field),
            _ => None,
        }
    }

    pub fn get_index(&self, index: usize) -> Option<Value> {
        match self {
            Value::List(values) => values.get(index).cloned(),
            Value::Mutable(inner) => inner.read().unwrap().get_index(index),
            _ => None,
        }
    }

    pub fn add(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left + right)),
            (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(left + right)),
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

        Err(ValueError::CannotAdd(self.clone(), other.clone()))
    }

    pub fn subtract(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Float(left), Value::Float(right)) => Ok(Value::Float(left - right)),
            (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(left - right)),
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
            (Value::Integer(left), Value::Integer(right)) => Ok(Value::Integer(left * right)),
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
                Ok(Value::Float((*left as f64) / (*right as f64)))
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
            (Value::Mutable(left), Value::Mutable(right)) => {
                let left = left.read().unwrap();
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
            (Value::Mutable(left), Value::Mutable(right)) => {
                let left = left.read().unwrap();
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
            (Value::Mutable(left), Value::Mutable(right)) => {
                let left = left.read().unwrap();
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
            (Value::Mutable(left), Value::Mutable(right)) => {
                let left = left.read().unwrap();
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
        match (self, other) {
            (Value::Boolean(left), Value::Boolean(right)) => Ok(Value::Boolean(*left && *right)),
            _ => Err(ValueError::CannotAnd(self.clone(), other.clone())),
        }
    }

    pub fn or(&self, other: &Value) -> Result<Value, ValueError> {
        match (self, other) {
            (Value::Boolean(left), Value::Boolean(right)) => Ok(Value::Boolean(*left || *right)),
            _ => Err(ValueError::CannotOr(self.clone(), other.clone())),
        }
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
            Value::Enum { name, r#type } => write!(f, "{name}::{type}"),
            Value::Float(float) => write!(f, "{float}"),
            Value::Function(function) => write!(f, "{function}"),
            Value::Integer(integer) => write!(f, "{integer}"),
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
            Value::Struct(structure) => write!(f, "{structure}"),
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
            Value::Enum { name, r#type } => {
                let mut ser = serializer.serialize_struct_variant("Value", 4, "Enum", 2)?;

                ser.serialize_field("name", name)?;
                ser.serialize_field("type", r#type)?;

                ser.end()
            }
            Value::Float(float) => serializer.serialize_f64(*float),
            Value::Function(function) => function.serialize(serializer),
            Value::Integer(integer) => serializer.serialize_i64(*integer),
            Value::List(list) => list.serialize(serializer),
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

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Function {
    pub name: Identifier,
    pub r#type: FunctionType,
    pub body: Arc<AbstractSyntaxTree>,
}

impl Function {
    pub fn call(
        &self,
        _type_arguments: Option<Vec<Type>>,
        value_arguments: Option<Vec<Value>>,
        context: &Context,
    ) -> Result<Option<Value>, VmError> {
        let new_context = Context::with_variables_from(context);

        if let (Some(value_parameters), Some(value_arguments)) =
            (&self.r#type.value_parameters, value_arguments)
        {
            for ((identifier, _), value) in value_parameters.iter().zip(value_arguments) {
                new_context.set_value(identifier.clone(), value);
            }
        }

        let mut vm = Vm::new(self.body.as_ref().clone(), new_context);

        vm.run()
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "fn {}", self.name)?;

        if let Some(type_parameters) = &self.r#type.type_parameters {
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

        if let Some(value_paramers) = &self.r#type.value_parameters {
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

impl Serialize for Function {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser = serializer.serialize_struct("Function", 3)?;

        ser.serialize_field("name", &self.name)?;
        ser.serialize_field("type", &self.r#type)?;
        ser.serialize_field("body", self.body.as_ref())?;

        ser.end()
    }
}

impl<'de> Deserialize<'de> for Function {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FunctionVisitor;

        impl<'de> Visitor<'de> for FunctionVisitor {
            type Value = Function;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a function")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Function, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut name = None;
                let mut r#type = None;
                let mut body = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "name" => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }

                            name = Some(map.next_value()?);
                        }
                        "type" => {
                            if r#type.is_some() {
                                return Err(de::Error::duplicate_field("type"));
                            }

                            r#type = Some(map.next_value()?);
                        }
                        "body" => {
                            if body.is_some() {
                                return Err(de::Error::duplicate_field("body"));
                            }

                            body = Some(map.next_value().map(|ast| Arc::new(ast))?);
                        }
                        _ => {
                            return Err(de::Error::unknown_field(key, &["name", "type", "body"]));
                        }
                    }
                }

                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let r#type = r#type.ok_or_else(|| de::Error::missing_field("type"))?;
                let body = body.ok_or_else(|| de::Error::missing_field("body"))?;

                Ok(Function { name, r#type, body })
            }
        }

        deserializer.deserialize_struct("Function", &["name", "type", "body"], FunctionVisitor)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Struct {
    Unit {
        r#type: Type,
    },
    Tuple {
        r#type: Type,
        fields: Vec<Value>,
    },
    Fields {
        r#type: Type,
        fields: Vec<(Identifier, Value)>,
    },
}

impl Display for Struct {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Struct::Unit { .. } => write!(f, "()"),
            Struct::Tuple { fields, .. } => {
                write!(f, "(")?;

                for (index, field) in fields.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}", field)?;
                }

                write!(f, ")")
            }
            Struct::Fields { fields, .. } => {
                write!(f, "{{ ")?;

                for (index, (identifier, r#type)) in fields.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}: {}", identifier, r#type)?;
                }

                write!(f, " }}")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
enum Rangeable {
    Byte(u8),
    Character(char),
    Float(f64),
    Integer(i64),
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
