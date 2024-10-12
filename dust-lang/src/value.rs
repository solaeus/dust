//! Dust value representation
//!
//! # Examples
//!
//! Each type of value has a corresponding method for instantiation:
//!
//! ```
//! # use dust_lang::Value;
//! let boolean = Value::boolean(true);
//! let float = Value::float(3.14);
//! let integer = Value::integer(42);
//! let string = Value::string("Hello, world!");
//! ```
//!
//! Values have a type, which can be retrieved using the `r#type` method:
//!
//! ```
//! # use dust_lang::*;
//! let value = Value::integer(42);
//!
//! assert_eq!(value.r#type(), Type::Integer);
//! ```
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter},
    ops::{Range, RangeInclusive},
};

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{Chunk, RangeableType, Span, Type, Vm, VmError};

/// Dust value representation
///
/// See the [module-level documentation][self] for more.
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Value {
    Primitive(Primitive),
    Object(Object),
}

impl Value {
    pub fn boolean(value: bool) -> Self {
        Value::Primitive(Primitive::Boolean(value))
    }

    pub fn byte(value: u8) -> Self {
        Value::Primitive(Primitive::Byte(value))
    }

    pub fn character(value: char) -> Self {
        Value::Primitive(Primitive::Character(value))
    }

    pub fn float(value: f64) -> Self {
        Value::Primitive(Primitive::Float(value))
    }

    pub fn function(body: Chunk) -> Self {
        Value::Primitive(Primitive::Function(Function { body }))
    }

    pub fn integer<T: Into<i64>>(into_i64: T) -> Self {
        Value::Primitive(Primitive::Integer(into_i64.into()))
    }

    pub fn list(start: u8, end: u8, item_type: Type) -> Self {
        Value::Object(Object::List {
            start,
            end,
            item_type,
        })
    }

    pub fn string<T: ToString>(to_string: T) -> Self {
        Value::Primitive(Primitive::String(to_string.to_string()))
    }

    pub fn r#type(&self) -> Type {
        match self {
            Value::Primitive(data) => data.r#type(),
            Value::Object(Object::List {
                start,
                end,
                item_type,
            }) => {
                let length = (end - start + 1) as usize;

                Type::List {
                    length,
                    item_type: Box::new(item_type.clone()),
                }
            }
        }
    }

    pub fn add(&self, other: &Value) -> Result<Value, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Primitive(left), Value::Primitive(right)) => (left, right),
            _ => return Err(ValueError::CannotAdd(self.clone(), other.clone())),
        };
        let sum = left
            .add(right)
            .ok_or_else(|| ValueError::CannotAdd(self.clone(), other.clone()))?;

        Ok(Value::Primitive(sum))
    }

    pub fn subtract(&self, other: &Value) -> Result<Value, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Primitive(left), Value::Primitive(right)) => (left, right),
            _ => return Err(ValueError::CannotAdd(self.clone(), other.clone())),
        };
        let difference = left
            .subtract(right)
            .ok_or_else(|| ValueError::CannotSubtract(self.clone(), other.clone()))?;

        Ok(Value::Primitive(difference))
    }

    pub fn multiply(&self, other: &Value) -> Result<Value, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Primitive(left), Value::Primitive(right)) => (left, right),
            _ => return Err(ValueError::CannotAdd(self.clone(), other.clone())),
        };
        let product = left
            .multiply(right)
            .ok_or_else(|| ValueError::CannotMultiply(self.clone(), other.clone()))?;

        Ok(Value::Primitive(product))
    }

    pub fn divide(&self, other: &Value) -> Result<Value, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Primitive(left), Value::Primitive(right)) => (left, right),
            _ => return Err(ValueError::CannotAdd(self.clone(), other.clone())),
        };
        let quotient = left
            .divide(right)
            .ok_or_else(|| ValueError::CannotDivide(self.clone(), other.clone()))?;

        Ok(Value::Primitive(quotient))
    }

    pub fn modulo(&self, other: &Value) -> Result<Value, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Primitive(left), Value::Primitive(right)) => (left, right),
            _ => return Err(ValueError::CannotAdd(self.clone(), other.clone())),
        };
        let remainder = left
            .modulo(right)
            .ok_or_else(|| ValueError::CannotModulo(self.clone(), other.clone()))?;

        Ok(Value::Primitive(remainder))
    }

    pub fn less_than(&self, other: &Value) -> Result<Value, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Primitive(left), Value::Primitive(right)) => (left, right),
            _ => return Err(ValueError::CannotAdd(self.clone(), other.clone())),
        };

        Ok(Value::boolean(left < right))
    }

    pub fn less_than_or_equal(&self, other: &Value) -> Result<Value, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Primitive(left), Value::Primitive(right)) => (left, right),
            _ => return Err(ValueError::CannotAdd(self.clone(), other.clone())),
        };

        left.less_than_or_equal(right)
            .ok_or_else(|| ValueError::CannotLessThanOrEqual(self.clone(), other.clone()))
            .map(Value::Primitive)
    }

    pub fn equal(&self, other: &Value) -> Result<Value, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Primitive(left), Value::Primitive(right)) => (left, right),
            _ => return Err(ValueError::CannotAdd(self.clone(), other.clone())),
        };

        Ok(Value::boolean(left == right))
    }

    pub fn not_equal(&self, other: &Value) -> Result<Value, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Primitive(left), Value::Primitive(right)) => (left, right),
            _ => return Err(ValueError::CannotAdd(self.clone(), other.clone())),
        };

        Ok(Value::boolean(left != right))
    }

    pub fn negate(&self) -> Result<Value, ValueError> {
        let data = match self {
            Value::Primitive(data) => data,
            _ => return Err(ValueError::CannotNot(self.clone())),
        };

        data.negate()
            .ok_or_else(|| ValueError::CannotNot(self.clone()))
            .map(Value::Primitive)
    }

    pub fn not(&self) -> Result<Value, ValueError> {
        let data = match self {
            Value::Primitive(data) => data,
            _ => return Err(ValueError::CannotNot(self.clone())),
        };

        data.not()
            .ok_or_else(|| ValueError::CannotNot(self.clone()))
            .map(Value::Primitive)
    }

    pub fn and(&self, other: &Value) -> Result<Value, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Primitive(left), Value::Primitive(right)) => (left, right),
            _ => return Err(ValueError::CannotAdd(self.clone(), other.clone())),
        };

        left.and(right)
            .ok_or_else(|| ValueError::CannotAnd(self.clone(), other.clone()))
            .map(Value::Primitive)
    }

    pub fn display(&self, vm: &Vm, position: Span) -> Result<String, ValueError> {
        match self {
            Value::Primitive(primitive) => Ok(primitive.to_string()),
            Value::Object(object) => object.display(vm, position),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::boolean(value)
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value::byte(value)
    }
}

impl From<char> for Value {
    fn from(value: char) -> Self {
        Value::character(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::float(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::integer(value as i64)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::integer(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::string(value)
    }
}

impl From<&str> for Value {
    fn from(str: &str) -> Self {
        Value::string(str)
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        log::trace!("Cloning value {:?}", self);

        match self {
            Value::Primitive(data) => Value::Primitive(data.clone()),
            Value::Object(object) => Value::Object(object.clone()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Primitive(primitive) => write!(f, "{primitive}"),
            Value::Object(_) => write!(f, "object"),
        }
    }
}

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Primitive(data) => data.serialize(serializer),
            Value::Object(object) => object.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a value")
            }

            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(Value::Primitive(Primitive::Boolean(value)))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(Value::Primitive(Primitive::Integer(value)))
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(Value::Primitive(Primitive::Integer(value as i64)))
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
                Ok(Value::Primitive(Primitive::Float(value)))
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(Value::Primitive(Primitive::String(value.to_string())))
            }

            fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(Value::Primitive(Primitive::String(value)))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Primitive {
    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Function(Function),
    Integer(i64),
    Range(RangeValue),
    String(String),
}

impl Primitive {
    pub fn r#type(&self) -> Type {
        match self {
            Primitive::Boolean(_) => Type::Boolean,
            Primitive::Byte(_) => Type::Byte,
            Primitive::Character(_) => Type::Character,
            Primitive::Function(Function { .. }) => todo!(),
            Primitive::Float(_) => Type::Float,
            Primitive::Integer(_) => Type::Integer,
            Primitive::Range(range) => range.r#type(),
            Primitive::String(string) => Type::String {
                length: Some(string.len()),
            },
        }
    }

    pub fn as_function(&self) -> Option<&Function> {
        if let Primitive::Function(function) = self {
            Some(function)
        } else {
            None
        }
    }

    pub fn is_rangeable(&self) -> bool {
        matches!(
            self,
            Primitive::Integer(_)
                | Primitive::Float(_)
                | Primitive::Character(_)
                | Primitive::Byte(_)
        )
    }

    pub fn add(&self, other: &Primitive) -> Option<Primitive> {
        match (self, other) {
            (Primitive::Byte(left), Primitive::Byte(right)) => {
                Some(Primitive::Byte(left.saturating_add(*right)))
            }
            (Primitive::Float(left), Primitive::Float(right)) => {
                Some(Primitive::Float(left + right))
            }
            (Primitive::Integer(left), Primitive::Integer(right)) => {
                Some(Primitive::Integer(left.saturating_add(*right)))
            }
            (Primitive::String(left), Primitive::String(right)) => {
                Some(Primitive::String(format!("{}{}", left, right)))
            }
            _ => None,
        }
    }

    pub fn subtract(&self, other: &Primitive) -> Option<Primitive> {
        match (self, other) {
            (Primitive::Byte(left), Primitive::Byte(right)) => {
                Some(Primitive::Byte(left.saturating_sub(*right)))
            }
            (Primitive::Float(left), Primitive::Float(right)) => {
                Some(Primitive::Float(left - right))
            }
            (Primitive::Integer(left), Primitive::Integer(right)) => {
                Some(Primitive::Integer(left.saturating_sub(*right)))
            }
            _ => None,
        }
    }

    pub fn multiply(&self, other: &Primitive) -> Option<Primitive> {
        match (self, other) {
            (Primitive::Byte(left), Primitive::Byte(right)) => {
                Some(Primitive::Byte(left.saturating_mul(*right)))
            }
            (Primitive::Float(left), Primitive::Float(right)) => {
                Some(Primitive::Float(left * right))
            }
            (Primitive::Integer(left), Primitive::Integer(right)) => {
                Some(Primitive::Integer(left.saturating_mul(*right)))
            }
            _ => None,
        }
    }

    pub fn divide(&self, other: &Primitive) -> Option<Primitive> {
        match (self, other) {
            (Primitive::Byte(left), Primitive::Byte(right)) => {
                Some(Primitive::Byte(left.saturating_div(*right)))
            }
            (Primitive::Float(left), Primitive::Float(right)) => {
                Some(Primitive::Float(left / right))
            }
            (Primitive::Integer(left), Primitive::Integer(right)) => {
                Some(Primitive::Integer(left.saturating_div(*right)))
            }
            _ => None,
        }
    }

    pub fn modulo(&self, other: &Primitive) -> Option<Primitive> {
        match (self, other) {
            (Primitive::Float(left), Primitive::Float(right)) => {
                Some(Primitive::Float(left % right))
            }
            (Primitive::Integer(left), Primitive::Integer(right)) => {
                Some(Primitive::Integer(left % right))
            }
            _ => None,
        }
    }

    pub fn less_than(&self, other: &Primitive) -> Option<Primitive> {
        match (self, other) {
            (Primitive::Float(left), Primitive::Float(right)) => {
                Some(Primitive::Boolean(left < right))
            }
            (Primitive::Integer(left), Primitive::Integer(right)) => {
                Some(Primitive::Boolean(left < right))
            }
            _ => None,
        }
    }

    pub fn less_than_or_equal(&self, other: &Primitive) -> Option<Primitive> {
        match (self, other) {
            (Primitive::Float(left), Primitive::Float(right)) => {
                Some(Primitive::Boolean(left <= right))
            }
            (Primitive::Integer(left), Primitive::Integer(right)) => {
                Some(Primitive::Boolean(left <= right))
            }
            _ => None,
        }
    }

    pub fn greater_than(&self, other: &Primitive) -> Option<Primitive> {
        match (self, other) {
            (Primitive::Float(left), Primitive::Float(right)) => {
                Some(Primitive::Boolean(left > right))
            }
            (Primitive::Integer(left), Primitive::Integer(right)) => {
                Some(Primitive::Boolean(left > right))
            }
            _ => None,
        }
    }

    pub fn greater_than_or_equal(&self, other: &Primitive) -> Option<Primitive> {
        match (self, other) {
            (Primitive::Float(left), Primitive::Float(right)) => {
                Some(Primitive::Boolean(left >= right))
            }
            (Primitive::Integer(left), Primitive::Integer(right)) => {
                Some(Primitive::Boolean(left >= right))
            }
            _ => None,
        }
    }

    pub fn and(&self, other: &Primitive) -> Option<Primitive> {
        match (self, other) {
            (Primitive::Boolean(left), Primitive::Boolean(right)) => {
                Some(Primitive::Boolean(*left && *right))
            }
            _ => None,
        }
    }

    pub fn or(&self, other: &Primitive) -> Option<Primitive> {
        match (self, other) {
            (Primitive::Boolean(left), Primitive::Boolean(right)) => {
                Some(Primitive::Boolean(*left || *right))
            }
            _ => None,
        }
    }

    pub fn is_even(&self) -> Option<Primitive> {
        match self {
            Primitive::Integer(integer) => Some(Primitive::Boolean(integer % 2 == 0)),
            Primitive::Float(float) => Some(Primitive::Boolean(float % 2.0 == 0.0)),
            _ => None,
        }
    }

    pub fn is_odd(&self) -> Option<Primitive> {
        match self {
            Primitive::Integer(integer) => Some(Primitive::Boolean(integer % 2 != 0)),
            Primitive::Float(float) => Some(Primitive::Boolean(float % 2.0 != 0.0)),
            _ => None,
        }
    }

    pub fn negate(&self) -> Option<Primitive> {
        match self {
            Primitive::Byte(value) => Some(Primitive::Byte(!value)),
            Primitive::Float(value) => Some(Primitive::Float(-value)),
            Primitive::Integer(value) => Some(Primitive::Integer(-value)),
            _ => None,
        }
    }

    pub fn not(&self) -> Option<Primitive> {
        match self {
            Primitive::Boolean(value) => Some(Primitive::Boolean(!value)),
            _ => None,
        }
    }
}

impl Display for Primitive {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Primitive::Boolean(boolean) => write!(f, "{boolean}"),
            Primitive::Byte(byte) => write!(f, "0x{byte:02x}"),
            Primitive::Character(character) => write!(f, "{character}"),
            Primitive::Float(float) => {
                write!(f, "{float}")?;

                if float.fract() == 0.0 {
                    write!(f, ".0")?;
                }

                Ok(())
            }
            Primitive::Function(Function { .. }) => {
                write!(f, "function")
            }
            Primitive::Integer(integer) => write!(f, "{integer}"),
            Primitive::Range(range_value) => {
                write!(f, "{range_value}")
            }
            Primitive::String(string) => write!(f, "{string}"),
        }
    }
}

impl Eq for Primitive {}

impl PartialOrd for Primitive {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Primitive {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Primitive::Boolean(left), Primitive::Boolean(right)) => left.cmp(right),
            (Primitive::Boolean(_), _) => Ordering::Greater,
            (Primitive::Byte(left), Primitive::Byte(right)) => left.cmp(right),
            (Primitive::Byte(_), _) => Ordering::Greater,
            (Primitive::Character(left), Primitive::Character(right)) => left.cmp(right),
            (Primitive::Character(_), _) => Ordering::Greater,
            (Primitive::Float(left), Primitive::Float(right)) => {
                if left.is_nan() && right.is_nan() {
                    Ordering::Equal
                } else if left.is_nan() {
                    Ordering::Less
                } else if right.is_nan() {
                    Ordering::Greater
                } else {
                    left.partial_cmp(right).unwrap()
                }
            }
            (Primitive::Float(_), _) => Ordering::Greater,
            (Primitive::Function(left), Primitive::Function(right)) => left.cmp(right),
            (Primitive::Function(_), _) => Ordering::Greater,
            (Primitive::Integer(left), Primitive::Integer(right)) => left.cmp(right),
            (Primitive::Integer(_), _) => Ordering::Greater,
            (Primitive::Range(left), Primitive::Range(right)) => left.cmp(right),
            (Primitive::Range(_), _) => Ordering::Greater,
            (Primitive::String(left), Primitive::String(right)) => left.cmp(right),
            (Primitive::String(_), _) => Ordering::Greater,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Function {
    pub body: Chunk,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RangeValue {
    ByteRange(Range<u8>),
    ByteRangeInclusive(RangeInclusive<u8>),
    CharacterRange(Range<char>),
    CharacterRangeInclusive(RangeInclusive<char>),
    FloatRange(Range<f64>),
    FloatRangeInclusive(RangeInclusive<f64>),
    IntegerRange(Range<i64>),
    IntegerRangeInclusive(RangeInclusive<i64>),
}

impl RangeValue {
    pub fn r#type(&self) -> Type {
        let inner_type = match self {
            RangeValue::ByteRange(_) => RangeableType::Byte,
            RangeValue::ByteRangeInclusive(_) => RangeableType::Byte,
            RangeValue::CharacterRange(_) => RangeableType::Character,
            RangeValue::CharacterRangeInclusive(_) => RangeableType::Character,
            RangeValue::FloatRange(_) => RangeableType::Float,
            RangeValue::FloatRangeInclusive(_) => RangeableType::Float,
            RangeValue::IntegerRange(_) => RangeableType::Integer,
            RangeValue::IntegerRangeInclusive(_) => RangeableType::Integer,
        };

        Type::Range { r#type: inner_type }
    }
}

impl From<Range<u8>> for RangeValue {
    fn from(range: Range<u8>) -> Self {
        RangeValue::ByteRange(range)
    }
}

impl From<RangeInclusive<u8>> for RangeValue {
    fn from(range: RangeInclusive<u8>) -> Self {
        RangeValue::ByteRangeInclusive(range)
    }
}

impl From<Range<char>> for RangeValue {
    fn from(range: Range<char>) -> Self {
        RangeValue::CharacterRange(range)
    }
}

impl From<RangeInclusive<char>> for RangeValue {
    fn from(range: RangeInclusive<char>) -> Self {
        RangeValue::CharacterRangeInclusive(range)
    }
}

impl From<Range<f64>> for RangeValue {
    fn from(range: Range<f64>) -> Self {
        RangeValue::FloatRange(range)
    }
}

impl From<RangeInclusive<f64>> for RangeValue {
    fn from(range: RangeInclusive<f64>) -> Self {
        RangeValue::FloatRangeInclusive(range)
    }
}

impl From<Range<i32>> for RangeValue {
    fn from(range: Range<i32>) -> Self {
        RangeValue::IntegerRange(range.start as i64..range.end as i64)
    }
}

impl From<RangeInclusive<i32>> for RangeValue {
    fn from(range: RangeInclusive<i32>) -> Self {
        RangeValue::IntegerRangeInclusive(*range.start() as i64..=*range.end() as i64)
    }
}

impl From<Range<i64>> for RangeValue {
    fn from(range: Range<i64>) -> Self {
        RangeValue::IntegerRange(range)
    }
}

impl From<RangeInclusive<i64>> for RangeValue {
    fn from(range: RangeInclusive<i64>) -> Self {
        RangeValue::IntegerRangeInclusive(range)
    }
}

impl Display for RangeValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            RangeValue::ByteRange(range) => write!(f, "{}..{}", range.start, range.end),
            RangeValue::ByteRangeInclusive(range) => {
                write!(f, "{}..={}", range.start(), range.end())
            }
            RangeValue::CharacterRange(range) => write!(f, "{}..{}", range.start, range.end),
            RangeValue::CharacterRangeInclusive(range) => {
                write!(f, "{}..={}", range.start(), range.end())
            }
            RangeValue::FloatRange(range) => write!(f, "{}..{}", range.start, range.end),
            RangeValue::FloatRangeInclusive(range) => {
                write!(f, "{}..={}", range.start(), range.end())
            }
            RangeValue::IntegerRange(range) => write!(f, "{}..{}", range.start, range.end),
            RangeValue::IntegerRangeInclusive(range) => {
                write!(f, "{}..={}", range.start(), range.end())
            }
        }
    }
}

impl Eq for RangeValue {}

impl PartialOrd for RangeValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RangeValue {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (RangeValue::ByteRange(left), RangeValue::ByteRange(right)) => {
                let start_cmp = left.start.cmp(&right.start);

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left.end.cmp(&right.end)
                }
            }
            (RangeValue::ByteRange(_), _) => Ordering::Greater,
            (RangeValue::ByteRangeInclusive(left), RangeValue::ByteRangeInclusive(right)) => {
                let start_cmp = left.start().cmp(right.start());

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left.end().cmp(right.end())
                }
            }
            (RangeValue::ByteRangeInclusive(_), _) => Ordering::Greater,
            (RangeValue::CharacterRange(left), RangeValue::CharacterRange(right)) => {
                let start_cmp = left.start.cmp(&right.start);

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left.end.cmp(&right.end)
                }
            }
            (RangeValue::CharacterRange(_), _) => Ordering::Greater,
            (
                RangeValue::CharacterRangeInclusive(left),
                RangeValue::CharacterRangeInclusive(right),
            ) => {
                let start_cmp = left.start().cmp(right.start());

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left.end().cmp(right.end())
                }
            }
            (RangeValue::CharacterRangeInclusive(_), _) => Ordering::Greater,
            (RangeValue::FloatRange(left), RangeValue::FloatRange(right)) => {
                let start_cmp = left.start.to_bits().cmp(&right.start.to_bits());

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left.end.to_bits().cmp(&right.end.to_bits())
                }
            }
            (RangeValue::FloatRange(_), _) => Ordering::Greater,
            (RangeValue::FloatRangeInclusive(left), RangeValue::FloatRangeInclusive(right)) => {
                let start_cmp = left.start().to_bits().cmp(&right.start().to_bits());

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left.end().to_bits().cmp(&right.end().to_bits())
                }
            }
            (RangeValue::FloatRangeInclusive(_), _) => Ordering::Greater,
            (RangeValue::IntegerRange(left), RangeValue::IntegerRange(right)) => {
                let start_cmp = left.start.cmp(&right.start);

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left.end.cmp(&right.end)
                }
            }
            (RangeValue::IntegerRange(_), _) => Ordering::Greater,
            (RangeValue::IntegerRangeInclusive(left), RangeValue::IntegerRangeInclusive(right)) => {
                let start_cmp = left.start().cmp(right.start());

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left.end().cmp(right.end())
                }
            }
            (RangeValue::IntegerRangeInclusive(_), _) => Ordering::Greater,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Object {
    List { start: u8, end: u8, item_type: Type },
}

impl Object {
    fn display(&self, vm: &Vm, position: Span) -> Result<String, ValueError> {
        match self {
            Object::List { start, end, .. } => {
                let mut display = String::from("[");
                let (start, end) = (*start, *end);

                for register in start..=end {
                    if register > start {
                        display.push_str(", ");
                    }

                    let value_display = match vm.get(register, position) {
                        Ok(value) => value.display(vm, position)?,
                        Err(error) => {
                            return Err(ValueError::CannotDisplay {
                                value: Value::Object(self.clone()),
                                vm_error: Box::new(error),
                            })
                        }
                    };

                    display.push_str(&value_display);
                }

                display.push(']');

                Ok(display)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueError {
    CannotAdd(Value, Value),
    CannotAnd(Value, Value),
    CannotDisplay {
        value: Value,
        vm_error: Box<VmError>,
    },
    CannotDivide(Value, Value),
    CannotLessThan(Value, Value),
    CannotLessThanOrEqual(Value, Value),
    CannotModulo(Value, Value),
    CannotMultiply(Value, Value),
    CannotNegate(Value),
    CannotNot(Value),
    CannotSubtract(Value, Value),
    CannotOr(Value, Value),
}

impl Display for ValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let get_value_display = |value: &Value| -> String {
            match value {
                Value::Primitive(primitive) => primitive.to_string(),
                Value::Object(_) => "Object".to_string(),
            }
        };

        match self {
            ValueError::CannotAdd(left, right) => {
                let left_display = get_value_display(left);
                let right_display = get_value_display(right);

                write!(f, "Cannot add {} and {}", left_display, right_display)
            }
            ValueError::CannotAnd(left, right) => {
                let left_display = get_value_display(left);
                let right_display = get_value_display(right);

                write!(
                    f,
                    "Cannot use logical and operation on {} and {}",
                    left_display, right_display
                )
            }
            ValueError::CannotDisplay { value, vm_error } => {
                let value_display = get_value_display(value);

                write!(f, "Cannot display {}: {:?}", value_display, vm_error)
            }
            ValueError::CannotDivide(left, right) => {
                let left_display = get_value_display(left);
                let right_display = get_value_display(right);

                write!(f, "Cannot divide {} by {}", left_display, right_display)
            }
            ValueError::CannotLessThan(left, right) => {
                let left_display = get_value_display(left);
                let right_display = get_value_display(right);

                write!(f, "Cannot compare {} and {}", left_display, right_display)
            }
            ValueError::CannotLessThanOrEqual(left, right) => {
                let left_display = get_value_display(left);
                let right_display = get_value_display(right);

                write!(f, "Cannot compare {} and {}", left_display, right_display)
            }
            ValueError::CannotModulo(left, right) => {
                let left_display = get_value_display(left);
                let right_display = get_value_display(right);

                write!(f, "Cannot modulo {} by {}", left_display, right_display)
            }
            ValueError::CannotMultiply(left, right) => {
                let left_display = get_value_display(left);
                let right_display = get_value_display(right);

                write!(f, "Cannot multiply {} by {}", left_display, right_display)
            }
            ValueError::CannotNegate(value) => {
                let value_display = get_value_display(value);

                write!(f, "Cannot negate {}", value_display)
            }
            ValueError::CannotNot(value) => {
                let value_display = get_value_display(value);

                write!(f, "Cannot use logical not operation on {}", value_display)
            }
            ValueError::CannotSubtract(left, right) => {
                let left_display = get_value_display(left);
                let right_display = get_value_display(right);

                write!(f, "Cannot subtract {} from {}", right_display, left_display)
            }
            ValueError::CannotOr(left, right) => {
                let left_display = get_value_display(left);
                let right_display = get_value_display(right);

                write!(
                    f,
                    "Cannot use logical or operation on {} and {}",
                    left_display, right_display
                )
            }
        }
    }
}
