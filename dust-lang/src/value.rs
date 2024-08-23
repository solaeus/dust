//! Dust value representation
use std::{
    cmp::Ordering,
    collections::HashMap,
    error::Error,
    fmt::{self, Display, Formatter},
    ops::{Range, RangeInclusive},
    sync::{Arc, RwLock},
};

use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{
    AbstractSyntaxTree, BuiltInFunction, BuiltInFunctionError, Context, ContextError, EnumType,
    FunctionType, Identifier, RangeableType, RuntimeError, StructType, Type, Vm,
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
/// # use dust_lang::*;
/// let value = Value::integer(42);
///
/// assert_eq!(value.r#type(), Type::Integer);
/// ```
#[derive(Debug, Clone)]
pub enum Value {
    Raw(ValueData),
    Reference(Arc<ValueData>),
    Mutable(Arc<RwLock<ValueData>>),
}

impl Value {
    pub fn boolean(value: bool) -> Self {
        Value::Raw(ValueData::Boolean(value))
    }

    pub fn byte(value: u8) -> Self {
        Value::Raw(ValueData::Byte(value))
    }

    pub fn character(value: char) -> Self {
        Value::Raw(ValueData::Character(value))
    }

    pub fn float(value: f64) -> Self {
        Value::Raw(ValueData::Float(value))
    }

    pub fn integer<T: Into<i64>>(into_i64: T) -> Self {
        Value::Raw(ValueData::Integer(into_i64.into()))
    }

    pub fn string<T: ToString>(to_string: T) -> Self {
        Value::Raw(ValueData::String(to_string.to_string()))
    }

    pub fn list(value: Vec<Value>) -> Self {
        Value::Raw(ValueData::List(value))
    }

    pub fn map<T: Into<HashMap<Identifier, Value>>>(into_map: T) -> Self {
        Value::Raw(ValueData::Map(into_map.into()))
    }

    pub fn mutable(value: Value) -> Self {
        match value {
            Value::Raw(data) => Value::Mutable(Arc::new(RwLock::new(data))),
            Value::Reference(data) => Value::Mutable(Arc::new(RwLock::new(data.as_ref().clone()))),
            Value::Mutable(_) => value,
        }
    }

    pub fn function(value: Function) -> Self {
        Value::Raw(ValueData::Function(value))
    }

    pub fn range<T: Into<RangeValue>>(range: T) -> Self {
        Value::Raw(ValueData::Range(range.into()))
    }

    pub fn r#struct(value: Struct) -> Self {
        Value::Raw(ValueData::Struct(value))
    }

    pub fn reference(value: Value) -> Self {
        match value {
            Value::Raw(data) => Value::Reference(Arc::new(data)),
            Value::Reference(_) => value,
            Value::Mutable(data) => {
                let data = data.read().unwrap();

                Value::Reference(Arc::new(data.clone()))
            }
        }
    }

    pub fn into_mutable(self) -> Self {
        match self {
            Value::Raw(data) => Value::Mutable(Arc::new(RwLock::new(data))),
            Value::Reference(data) => Value::Mutable(Arc::new(RwLock::new(data.as_ref().clone()))),
            Value::Mutable(_) => self,
        }
    }

    pub fn into_reference(self) -> Self {
        match self {
            Value::Raw(data) => Value::Reference(Arc::new(data)),
            Value::Reference(_) => self,
            Value::Mutable(data) => {
                let data = data.read().unwrap();

                Value::Reference(Arc::new(data.clone()))
            }
        }
    }

    pub fn clone_data(&self) -> ValueData {
        match self {
            Value::Raw(data) => data.clone(),
            Value::Reference(data) => data.as_ref().clone(),
            Value::Mutable(data) => data.read().unwrap().clone(),
        }
    }

    pub fn is_rangeable(&self) -> bool {
        match self {
            Value::Raw(data) => data.is_rangeable(),
            Value::Reference(data) => data.is_rangeable(),
            Value::Mutable(data) => data.read().unwrap().is_rangeable(),
        }
    }

    pub fn as_mutable(&self) -> Option<&Arc<RwLock<ValueData>>> {
        match self {
            Value::Mutable(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Raw(ValueData::Boolean(value)) => Some(*value),
            Value::Reference(data) => match data.as_ref() {
                ValueData::Boolean(value) => Some(*value),
                _ => None,
            },
            Value::Mutable(data) => match *data.read().unwrap() {
                ValueData::Boolean(value) => Some(value),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn as_byte(&self) -> Option<u8> {
        match self {
            Value::Raw(ValueData::Byte(value)) => Some(*value),
            Value::Reference(data) => match data.as_ref() {
                ValueData::Byte(value) => Some(*value),
                _ => None,
            },
            Value::Mutable(data) => match *data.read().unwrap() {
                ValueData::Byte(value) => Some(value),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn as_character(&self) -> Option<char> {
        match self {
            Value::Raw(ValueData::Character(value)) => Some(*value),
            Value::Reference(data) => match data.as_ref() {
                ValueData::Character(value) => Some(*value),
                _ => None,
            },
            Value::Mutable(data) => match *data.read().unwrap() {
                ValueData::Character(value) => Some(value),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Raw(ValueData::Float(value)) => Some(*value),
            Value::Reference(data) => match data.as_ref() {
                ValueData::Float(value) => Some(*value),
                _ => None,
            },
            Value::Mutable(data) => match *data.read().unwrap() {
                ValueData::Float(value) => Some(value),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Raw(ValueData::Integer(value)) => Some(*value),
            Value::Reference(data) => match data.as_ref() {
                ValueData::Integer(value) => Some(*value),
                _ => None,
            },
            Value::Mutable(data) => match *data.read().unwrap() {
                ValueData::Integer(value) => Some(value),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn r#type(&self) -> Type {
        match self {
            Value::Raw(data) => data.r#type(),
            Value::Reference(data) => data.r#type(),
            Value::Mutable(data) => data.read().unwrap().r#type(),
        }
    }

    pub fn mutate(&self, value: Value) -> Result<(), ValueError> {
        match self {
            Value::Mutable(data) => {
                let mut data = data.write().unwrap();
                *data = value.clone_data();

                Ok(())
            }
            _ => Err(ValueError::CannotMutate(self.clone())),
        }
    }

    pub fn index(&self, index_value: &Value) -> Result<Value, ValueError> {
        let collection = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let index = match index_value {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };

        match (collection, index) {
            (ValueData::List(values), ValueData::Integer(index)) => values
                .get(*index as usize)
                .cloned()
                .ok_or_else(|| ValueError::IndexOutOfBounds {
                    value: self.clone(),
                    index: *index,
                }),
            (ValueData::List(values), ValueData::Range(RangeValue::IntegerRange(range))) => {
                if range.start < 0 || range.start > values.len() as i64 {
                    return Err(ValueError::IndexOutOfBounds {
                        value: self.clone(),
                        index: range.start,
                    });
                }

                if range.end < 0 || range.end > values.len() as i64 {
                    return Err(ValueError::IndexOutOfBounds {
                        value: self.clone(),
                        index: range.end,
                    });
                }

                let slice = values
                    .get(range.start as usize..range.end as usize)
                    .unwrap();

                Ok(Value::list(slice.to_vec()))
            }
            (ValueData::String(string), ValueData::Integer(index)) => {
                let index = *index as usize;
                let character =
                    string
                        .chars()
                        .nth(index)
                        .ok_or_else(|| ValueError::IndexOutOfBounds {
                            value: self.clone(),
                            index: index as i64,
                        })?;

                Ok(Value::character(character))
            }
            _ => Err(ValueError::CannotIndex {
                value: self.clone(),
                index: index_value.clone(),
            }),
        }
    }

    pub fn get_field(&self, field: &Identifier) -> Option<Value> {
        let data = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };

        match field.as_str() {
            "is_even" => match data {
                ValueData::Integer(integer) => Some(Value::boolean(integer % 2 == 0)),
                ValueData::Float(float) => Some(Value::boolean(float % 2.0 == 0.0)),
                _ => None,
            },
            "is_odd" => match data {
                ValueData::Integer(integer) => Some(Value::boolean(integer % 2 != 0)),
                ValueData::Float(float) => Some(Value::boolean(float % 2.0 != 0.0)),
                _ => None,
            },
            "to_string" => Some(Value::function(Function::BuiltIn(
                BuiltInFunction::ToString,
            ))),
            "length" => match data {
                ValueData::List(values) => Some(Value::integer(values.len() as i64)),
                ValueData::String(string) => Some(Value::integer(string.len() as i64)),
                ValueData::Map(map) => Some(Value::integer(map.len() as i64)),
                _ => None,
            },
            _ => match data {
                ValueData::Struct(Struct::Fields { fields, .. }) => fields.get(field).cloned(),
                ValueData::Map(pairs) => pairs.get(field).cloned(),
                _ => None,
            },
        }
    }

    pub fn add(&self, other: &Value) -> Result<Value, ValueError> {
        let left = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let sum = left
            .add(right)
            .ok_or_else(|| ValueError::CannotAdd(self.clone(), other.clone()))?;

        Ok(Value::Raw(sum))
    }

    pub fn add_assign(&self, other: &Value) -> Result<(), ValueError> {
        let mut left = self
            .as_mutable()
            .ok_or(ValueError::CannotMutate(self.clone()))?
            .write()
            .unwrap();
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let new_data = left
            .add(right)
            .ok_or_else(|| ValueError::CannotAdd(self.clone(), other.clone()))?;

        *left = new_data;

        Ok(())
    }

    pub fn subtract(&self, other: &Value) -> Result<Value, ValueError> {
        let left = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let difference = left
            .subtract(right)
            .ok_or_else(|| ValueError::CannotSubtract(self.clone(), other.clone()))?;

        Ok(Value::Raw(difference))
    }

    pub fn subtract_assign(&self, other: &Value) -> Result<(), ValueError> {
        let mut left = self
            .as_mutable()
            .ok_or(ValueError::CannotMutate(self.clone()))?
            .write()
            .unwrap();
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let new_data = left
            .subtract(right)
            .ok_or_else(|| ValueError::CannotSubtract(self.clone(), other.clone()))?;

        *left = new_data;

        Ok(())
    }

    pub fn multiply(&self, other: &Value) -> Result<Value, ValueError> {
        let left = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let product = left
            .multiply(right)
            .ok_or_else(|| ValueError::CannotMultiply(self.clone(), other.clone()))?;

        Ok(Value::Raw(product))
    }

    pub fn multiply_assign(&self, other: &Value) -> Result<(), ValueError> {
        let mut left = self
            .as_mutable()
            .ok_or(ValueError::CannotMutate(self.clone()))?
            .write()
            .unwrap();
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let new_data = left
            .multiply(right)
            .ok_or_else(|| ValueError::CannotMultiply(self.clone(), other.clone()))?;

        *left = new_data;

        Ok(())
    }

    pub fn divide(&self, other: &Value) -> Result<Value, ValueError> {
        let left = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let quotient = left
            .divide(right)
            .ok_or_else(|| ValueError::CannotDivide(self.clone(), other.clone()))?;

        Ok(Value::Raw(quotient))
    }

    pub fn divide_assign(&self, other: &Value) -> Result<(), ValueError> {
        let mut left = self
            .as_mutable()
            .ok_or(ValueError::CannotMutate(self.clone()))?
            .write()
            .unwrap();
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let new_data = left
            .divide(right)
            .ok_or_else(|| ValueError::CannotDivide(self.clone(), other.clone()))?;

        *left = new_data;

        Ok(())
    }

    pub fn modulo(&self, other: &Value) -> Result<Value, ValueError> {
        let left = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let remainder = left
            .modulo(right)
            .ok_or_else(|| ValueError::CannotModulo(self.clone(), other.clone()))?;

        Ok(Value::Raw(remainder))
    }

    pub fn modulo_assign(&self, other: &Value) -> Result<(), ValueError> {
        let mut left = self
            .as_mutable()
            .ok_or(ValueError::CannotMutate(self.clone()))?
            .write()
            .unwrap();
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let new_data = left
            .modulo(right)
            .ok_or_else(|| ValueError::CannotModulo(self.clone(), other.clone()))?;

        *left = new_data;

        Ok(())
    }

    pub fn is_even(&self) -> Result<Value, ValueError> {
        let data = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };

        data.is_even()
            .ok_or_else(|| ValueError::CannotModulo(self.clone(), Value::integer(2)))
            .map(Value::Raw)
    }

    pub fn is_odd(&self) -> Result<Value, ValueError> {
        let data = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };

        data.is_odd()
            .ok_or_else(|| ValueError::CannotModulo(self.clone(), Value::integer(2)))
            .map(Value::Raw)
    }

    pub fn greater_than(&self, other: &Value) -> Result<Value, ValueError> {
        let left = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };

        left.greater_than(right)
            .ok_or_else(|| ValueError::CannotGreaterThan(self.clone(), other.clone()))
            .map(Value::Raw)
    }

    pub fn greater_than_or_equal(&self, other: &Value) -> Result<Value, ValueError> {
        let left = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };

        left.greater_than_or_equal(right)
            .ok_or_else(|| ValueError::CannotGreaterThanOrEqual(self.clone(), other.clone()))
            .map(Value::Raw)
    }

    pub fn less_than(&self, other: &Value) -> Result<Value, ValueError> {
        let left = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };

        left.less_than(right)
            .ok_or_else(|| ValueError::CannotLessThan(self.clone(), other.clone()))
            .map(Value::Raw)
    }

    pub fn less_than_or_equal(&self, other: &Value) -> Result<Value, ValueError> {
        let left = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };

        left.less_than_or_equal(right)
            .ok_or_else(|| ValueError::CannotLessThanOrEqual(self.clone(), other.clone()))
            .map(Value::Raw)
    }

    pub fn equal(&self, other: &Value) -> Result<Value, ValueError> {
        let left = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };

        Ok(Value::boolean(left == right))
    }

    pub fn not_equal(&self, other: &Value) -> Result<Value, ValueError> {
        let left = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };

        Ok(Value::boolean(left != right))
    }

    pub fn negate(&self) -> Result<Value, ValueError> {
        let data = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };

        data.negate()
            .ok_or_else(|| ValueError::CannotNegate(self.clone()))
            .map(Value::Raw)
    }

    pub fn not(&self) -> Result<Value, ValueError> {
        let data = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };

        data.not()
            .ok_or_else(|| ValueError::CannotNot(self.clone()))
            .map(Value::Raw)
    }

    pub fn and(&self, other: &Value) -> Result<Value, ValueError> {
        let left = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };

        left.and(right)
            .ok_or_else(|| ValueError::CannotAnd(self.clone(), other.clone()))
            .map(Value::Raw)
    }

    pub fn or(&self, other: &Value) -> Result<Value, ValueError> {
        let left = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };

        left.or(right)
            .ok_or_else(|| ValueError::CannotOr(self.clone(), other.clone()))
            .map(Value::Raw)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Raw(data) => write!(f, "{}", data),
            Value::Reference(data) => write!(f, "{}", data),
            Value::Mutable(data) => write!(f, "{}", data.read().unwrap()),
        }
    }
}

impl Eq for Value {}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        let left = match self {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };
        let right = match other {
            Value::Raw(data) => data,
            Value::Reference(data) => data,
            Value::Mutable(data) => &data.read().unwrap(),
        };

        left == right
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
            (Value::Raw(left), Value::Raw(right)) => left.cmp(right),
            (Value::Raw(_), _) => Ordering::Greater,
            (Value::Reference(left), Value::Reference(right)) => left.cmp(right),
            (Value::Reference(_), _) => Ordering::Greater,
            (Value::Mutable(left), Value::Mutable(right)) => {
                left.read().unwrap().cmp(&right.read().unwrap())
            }
            (Value::Mutable(_), _) => Ordering::Greater,
        }
    }
}

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Raw(data) => data.serialize(serializer),
            Value::Reference(data) => data.serialize(serializer),
            Value::Mutable(data) => data.read().unwrap().serialize(serializer),
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
                Ok(Value::Raw(ValueData::Boolean(value)))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(Value::Raw(ValueData::Integer(value)))
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(Value::Raw(ValueData::Integer(value as i64)))
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
                Ok(Value::Raw(ValueData::Float(value)))
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(Value::Raw(ValueData::String(value.to_string())))
            }

            fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(Value::Raw(ValueData::String(value)))
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();

                while let Some(value) = seq.next_element()? {
                    values.push(value);
                }

                Ok(Value::Raw(ValueData::List(values)))
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut values = HashMap::new();

                while let Some((key, value)) = map.next_entry()? {
                    values.insert(key, value);
                }

                Ok(Value::Raw(ValueData::Map(values)))
            }

            fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
                Ok(Value::Raw(ValueData::List(
                    value.iter().map(|&byte| Value::byte(byte)).collect(),
                )))
            }

            fn visit_byte_buf<E: de::Error>(self, value: Vec<u8>) -> Result<Self::Value, E> {
                Ok(Value::Raw(ValueData::List(
                    value.iter().map(|&byte| Value::byte(byte)).collect(),
                )))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

#[derive(Clone, Debug)]
pub enum ValueData {
    Boolean(bool),
    Byte(u8),
    Character(char),
    Enum(Enum),
    Float(f64),
    Function(Function),
    Integer(i64),
    List(Vec<Value>),
    Map(HashMap<Identifier, Value>),
    Range(RangeValue),
    String(String),
    Struct(Struct),
    Tuple(Vec<Value>),
}

impl ValueData {
    pub fn r#type(&self) -> Type {
        match self {
            ValueData::Boolean(_) => Type::Boolean,
            ValueData::Byte(_) => Type::Byte,
            ValueData::Character(_) => Type::Character,
            ValueData::Enum(Enum { r#type, .. }) => Type::Enum(r#type.clone()),
            ValueData::Float(_) => Type::Float,
            ValueData::Function(Function::BuiltIn(built_in_function)) => {
                Type::Function(FunctionType {
                    name: Identifier::new(built_in_function.name()),
                    type_parameters: built_in_function.type_parameters(),
                    value_parameters: built_in_function.value_parameters(),
                    return_type: built_in_function.return_type().map(Box::new),
                })
            }
            ValueData::Function(Function::Parsed { r#type, .. }) => Type::Function(r#type.clone()),
            ValueData::Integer(_) => Type::Integer,
            ValueData::List(values) => {
                let item_type = values.first().unwrap().r#type();

                Type::List {
                    item_type: Box::new(item_type),
                    length: values.len(),
                }
            }
            ValueData::Map(map) => {
                let pairs = map
                    .iter()
                    .map(|(key, value)| (key.clone(), value.r#type()))
                    .collect();

                Type::Map { pairs }
            }
            ValueData::Range(range) => range.r#type(),
            ValueData::String(string) => Type::String {
                length: Some(string.len()),
            },
            ValueData::Struct(r#struct) => match r#struct {
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
            ValueData::Tuple(values) => {
                let fields = values.iter().map(|value| value.r#type()).collect();

                Type::Tuple(fields)
            }
        }
    }

    pub fn is_rangeable(&self) -> bool {
        matches!(
            self,
            ValueData::Integer(_)
                | ValueData::Float(_)
                | ValueData::Character(_)
                | ValueData::Byte(_)
        )
    }

    pub fn add(&self, other: &ValueData) -> Option<ValueData> {
        match (self, other) {
            (ValueData::Float(left), ValueData::Float(right)) => {
                Some(ValueData::Float(left + right))
            }
            (ValueData::Integer(left), ValueData::Integer(right)) => {
                Some(ValueData::Integer(left.saturating_add(*right)))
            }
            (ValueData::String(left), ValueData::String(right)) => {
                Some(ValueData::String(format!("{}{}", left, right)))
            }
            _ => None,
        }
    }

    pub fn subtract(&self, other: &ValueData) -> Option<ValueData> {
        match (self, other) {
            (ValueData::Float(left), ValueData::Float(right)) => {
                Some(ValueData::Float(left - right))
            }
            (ValueData::Integer(left), ValueData::Integer(right)) => {
                Some(ValueData::Integer(left.saturating_sub(*right)))
            }
            _ => None,
        }
    }

    pub fn multiply(&self, other: &ValueData) -> Option<ValueData> {
        match (self, other) {
            (ValueData::Float(left), ValueData::Float(right)) => {
                Some(ValueData::Float(left * right))
            }
            (ValueData::Integer(left), ValueData::Integer(right)) => {
                Some(ValueData::Integer(left.saturating_mul(*right)))
            }
            _ => None,
        }
    }

    pub fn divide(&self, other: &ValueData) -> Option<ValueData> {
        match (self, other) {
            (ValueData::Float(left), ValueData::Float(right)) => {
                Some(ValueData::Float(left / right))
            }
            (ValueData::Integer(left), ValueData::Integer(right)) => {
                Some(ValueData::Integer(left.saturating_div(*right)))
            }
            _ => None,
        }
    }

    pub fn modulo(&self, other: &ValueData) -> Option<ValueData> {
        match (self, other) {
            (ValueData::Float(left), ValueData::Float(right)) => {
                Some(ValueData::Float(left % right))
            }
            (ValueData::Integer(left), ValueData::Integer(right)) => {
                Some(ValueData::Integer(left % right))
            }
            _ => None,
        }
    }

    pub fn less_than(&self, other: &ValueData) -> Option<ValueData> {
        match (self, other) {
            (ValueData::Float(left), ValueData::Float(right)) => {
                Some(ValueData::Boolean(left < right))
            }
            (ValueData::Integer(left), ValueData::Integer(right)) => {
                Some(ValueData::Boolean(left < right))
            }
            _ => None,
        }
    }

    pub fn less_than_or_equal(&self, other: &ValueData) -> Option<ValueData> {
        match (self, other) {
            (ValueData::Float(left), ValueData::Float(right)) => {
                Some(ValueData::Boolean(left <= right))
            }
            (ValueData::Integer(left), ValueData::Integer(right)) => {
                Some(ValueData::Boolean(left <= right))
            }
            _ => None,
        }
    }

    pub fn greater_than(&self, other: &ValueData) -> Option<ValueData> {
        match (self, other) {
            (ValueData::Float(left), ValueData::Float(right)) => {
                Some(ValueData::Boolean(left > right))
            }
            (ValueData::Integer(left), ValueData::Integer(right)) => {
                Some(ValueData::Boolean(left > right))
            }
            _ => None,
        }
    }

    pub fn greater_than_or_equal(&self, other: &ValueData) -> Option<ValueData> {
        match (self, other) {
            (ValueData::Float(left), ValueData::Float(right)) => {
                Some(ValueData::Boolean(left >= right))
            }
            (ValueData::Integer(left), ValueData::Integer(right)) => {
                Some(ValueData::Boolean(left >= right))
            }
            _ => None,
        }
    }

    pub fn and(&self, other: &ValueData) -> Option<ValueData> {
        match (self, other) {
            (ValueData::Boolean(left), ValueData::Boolean(right)) => {
                Some(ValueData::Boolean(*left && *right))
            }
            _ => None,
        }
    }

    pub fn or(&self, other: &ValueData) -> Option<ValueData> {
        match (self, other) {
            (ValueData::Boolean(left), ValueData::Boolean(right)) => {
                Some(ValueData::Boolean(*left || *right))
            }
            _ => None,
        }
    }

    pub fn is_even(&self) -> Option<ValueData> {
        match self {
            ValueData::Integer(integer) => Some(ValueData::Boolean(integer % 2 == 0)),
            ValueData::Float(float) => Some(ValueData::Boolean(float % 2.0 == 0.0)),
            _ => None,
        }
    }

    pub fn is_odd(&self) -> Option<ValueData> {
        match self {
            ValueData::Integer(integer) => Some(ValueData::Boolean(integer % 2 != 0)),
            ValueData::Float(float) => Some(ValueData::Boolean(float % 2.0 != 0.0)),
            _ => None,
        }
    }

    pub fn negate(&self) -> Option<ValueData> {
        match self {
            ValueData::Byte(value) => Some(ValueData::Byte(!value)),
            ValueData::Float(value) => Some(ValueData::Float(-value)),
            ValueData::Integer(value) => Some(ValueData::Integer(-value)),
            _ => None,
        }
    }

    pub fn not(&self) -> Option<ValueData> {
        match self {
            ValueData::Boolean(value) => Some(ValueData::Boolean(!value)),
            _ => None,
        }
    }
}

impl From<bool> for ValueData {
    fn from(value: bool) -> Self {
        ValueData::Boolean(value)
    }
}

impl From<u8> for ValueData {
    fn from(value: u8) -> Self {
        ValueData::Byte(value)
    }
}

impl From<char> for ValueData {
    fn from(value: char) -> Self {
        ValueData::Character(value)
    }
}

impl From<f64> for ValueData {
    fn from(value: f64) -> Self {
        ValueData::Float(value)
    }
}

impl From<i32> for ValueData {
    fn from(value: i32) -> Self {
        ValueData::Integer(value as i64)
    }
}

impl From<i64> for ValueData {
    fn from(value: i64) -> Self {
        ValueData::Integer(value)
    }
}

impl From<String> for ValueData {
    fn from(value: String) -> Self {
        ValueData::String(value)
    }
}

impl From<&str> for ValueData {
    fn from(value: &str) -> Self {
        ValueData::String(value.to_string())
    }
}

impl Display for ValueData {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ValueData::Boolean(boolean) => write!(f, "{boolean}"),
            ValueData::Byte(byte) => write!(f, "{byte}"),
            ValueData::Character(character) => write!(f, "{character}"),
            ValueData::Enum(r#enum) => write!(f, "{enum}"),
            ValueData::Float(float) => write!(f, "{float}"),
            ValueData::Function(function) => write!(f, "{function}"),
            ValueData::Integer(integer) => write!(f, "{integer}"),
            ValueData::Map(pairs) => {
                write!(f, "{{ ")?;

                for (index, (key, value)) in pairs.iter().enumerate() {
                    write!(f, "{key}: {value}")?;

                    if index < pairs.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, " }}")
            }
            ValueData::List(list) => {
                write!(f, "[")?;

                for (index, value) in list.iter().enumerate() {
                    write!(f, "{}", value)?;

                    if index < list.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, "]")
            }
            ValueData::Range(range_value) => {
                write!(f, "{range_value}")
            }
            ValueData::String(string) => write!(f, "{string}"),
            ValueData::Struct(r#struct) => write!(f, "{struct}"),
            ValueData::Tuple(fields) => {
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

impl Eq for ValueData {}

impl PartialEq for ValueData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ValueData::Boolean(left), ValueData::Boolean(right)) => left == right,
            (ValueData::Byte(left), ValueData::Byte(right)) => left == right,
            (ValueData::Character(left), ValueData::Character(right)) => left == right,
            (ValueData::Float(left), ValueData::Float(right)) => left == right,
            (ValueData::Function(left), ValueData::Function(right)) => left == right,
            (ValueData::Integer(left), ValueData::Integer(right)) => left == right,
            (ValueData::List(left), ValueData::List(right)) => left == right,
            (ValueData::Map(left), ValueData::Map(right)) => left == right,
            (ValueData::Range(left), ValueData::Range(right)) => left == right,
            (ValueData::String(left), ValueData::String(right)) => left == right,
            (ValueData::Struct(left), ValueData::Struct(right)) => left == right,
            (ValueData::Tuple(left), ValueData::Tuple(right)) => left == right,
            _ => false,
        }
    }
}

impl PartialOrd for ValueData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValueData {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (ValueData::Boolean(left), ValueData::Boolean(right)) => left.cmp(right),
            (ValueData::Boolean(_), _) => Ordering::Greater,
            (ValueData::Byte(left), ValueData::Byte(right)) => left.cmp(right),
            (ValueData::Byte(_), _) => Ordering::Greater,
            (ValueData::Character(left), ValueData::Character(right)) => left.cmp(right),
            (ValueData::Character(_), _) => Ordering::Greater,
            (ValueData::Float(left), ValueData::Float(right)) => left.partial_cmp(right).unwrap(),
            (ValueData::Float(_), _) => Ordering::Greater,
            (ValueData::Function(left), ValueData::Function(right)) => left.cmp(right),
            (ValueData::Function(_), _) => Ordering::Greater,
            (ValueData::Integer(left), ValueData::Integer(right)) => left.cmp(right),
            (ValueData::Integer(_), _) => Ordering::Greater,
            (ValueData::List(left), ValueData::List(right)) => left.cmp(right),
            (ValueData::List(_), _) => Ordering::Greater,
            (ValueData::Range(left), ValueData::Range(right)) => left.cmp(right),
            (ValueData::Range(_), _) => Ordering::Greater,
            (ValueData::String(left), ValueData::String(right)) => left.cmp(right),
            (ValueData::String(_), _) => Ordering::Greater,
            (ValueData::Struct(left), ValueData::Struct(right)) => left.cmp(right),
            (ValueData::Struct(_), _) => Ordering::Greater,
            (ValueData::Tuple(left), ValueData::Tuple(right)) => left.cmp(right),
            _ => Ordering::Greater,
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
            ValueData::Byte(byte) => serializer.serialize_u8(*byte),
            ValueData::Character(character) => serializer.serialize_char(*character),
            ValueData::Enum(r#emum) => r#emum.serialize(serializer),
            ValueData::Float(float) => serializer.serialize_f64(*float),
            ValueData::Function(function) => function.serialize(serializer),
            ValueData::Integer(integer) => serializer.serialize_i64(*integer),
            ValueData::List(list) => list.serialize(serializer),
            ValueData::Map(pairs) => {
                let mut ser = serializer.serialize_map(Some(pairs.len()))?;

                for (key, value) in pairs {
                    ser.serialize_entry(key, value)?;
                }

                ser.end()
            }
            ValueData::Range(range) => range.serialize(serializer),
            ValueData::String(string) => serializer.serialize_str(string),
            ValueData::Struct(r#struct) => r#struct.serialize(serializer),
            ValueData::Tuple(tuple) => tuple.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for ValueData {
    fn deserialize<D>(deserializer: D) -> Result<ValueData, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueDataVisitor;

        impl<'de> Visitor<'de> for ValueDataVisitor {
            type Value = ValueData;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<ValueData, E> {
                Ok(ValueData::Boolean(value))
            }

            fn visit_u8<E>(self, value: u8) -> Result<ValueData, E> {
                Ok(ValueData::Byte(value))
            }

            fn visit_char<E>(self, value: char) -> Result<ValueData, E> {
                Ok(ValueData::Character(value))
            }

            fn visit_f64<E>(self, value: f64) -> Result<ValueData, E> {
                Ok(ValueData::Float(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<ValueData, E> {
                Ok(ValueData::Integer(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<ValueData, E> {
                Ok(ValueData::String(value.to_string()))
            }
        }

        deserializer.deserialize_any(ValueDataVisitor)
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
    ) -> Result<Option<Value>, FunctionCallError> {
        match self {
            Function::BuiltIn(built_in_function) => built_in_function
                .call(_type_arguments, value_arguments)
                .map_err(FunctionCallError::BuiltInFunction),
            Function::Parsed { r#type, body, .. } => {
                let new_context =
                    Context::with_data_from(context).map_err(FunctionCallError::Context)?;

                if let (Some(value_parameters), Some(value_arguments)) =
                    (&r#type.value_parameters, value_arguments)
                {
                    for ((identifier, _), value) in value_parameters.iter().zip(value_arguments) {
                        new_context
                            .set_variable_value(identifier.clone(), value)
                            .map_err(FunctionCallError::Context)?;
                    }
                }

                let mut vm = Vm::new(body, new_context);

                vm.run()
                    .map_err(|error| FunctionCallError::Runtime(Box::new(error)))
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

#[derive(Clone, Debug, PartialEq)]
pub enum FunctionCallError {
    BuiltInFunction(BuiltInFunctionError),
    Context(ContextError),
    Runtime(Box<RuntimeError>),
}

impl Display for FunctionCallError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            FunctionCallError::BuiltInFunction(error) => write!(f, "{}", error),
            FunctionCallError::Context(error) => write!(f, "{}", error),
            FunctionCallError::Runtime(error) => write!(f, "{}", error),
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

                left_fields.iter().cmp(right_fields.iter())
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
        }
    }
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
    CannotNegate(Value),
    CannotNot(Value),
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
            ValueError::CannotNegate(value) => write!(f, "Cannot negate {}", value),
            ValueError::CannotNot(value) => {
                write!(f, "Cannot use logical not operation on {}", value)
            }
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
