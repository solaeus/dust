use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use tracing::trace;

use crate::{Chunk, Type, Value};

use super::RangeValue;

pub type DustString = SmartString<LazyCompact>;

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ConcreteValue {
    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Function(Arc<Chunk>),
    Integer(i64),
    List(Vec<ConcreteValue>),
    Range(RangeValue),
    String(DustString),
}

impl ConcreteValue {
    pub fn to_value(self) -> Value {
        Value::Concrete(self)
    }

    pub fn list<T: Into<Vec<ConcreteValue>>>(into_items: T) -> Self {
        ConcreteValue::List(into_items.into())
    }

    pub fn string<T: Into<SmartString<LazyCompact>>>(to_string: T) -> Self {
        ConcreteValue::String(to_string.into())
    }

    pub fn as_boolean(&self) -> Option<&bool> {
        if let ConcreteValue::Boolean(boolean) = self {
            Some(boolean)
        } else {
            None
        }
    }

    pub fn as_byte(&self) -> Option<&u8> {
        if let ConcreteValue::Byte(byte) = self {
            Some(byte)
        } else {
            None
        }
    }

    pub fn as_character(&self) -> Option<&char> {
        if let ConcreteValue::Character(character) = self {
            Some(character)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<&f64> {
        if let ConcreteValue::Float(float) = self {
            Some(float)
        } else {
            None
        }
    }

    pub fn as_integer(&self) -> Option<&i64> {
        if let ConcreteValue::Integer(integer) = self {
            Some(integer)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&DustString> {
        if let ConcreteValue::String(string) = self {
            Some(string)
        } else {
            None
        }
    }

    pub fn as_list(&self) -> Option<&Vec<ConcreteValue>> {
        if let ConcreteValue::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    pub fn as_range(&self) -> Option<&RangeValue> {
        if let ConcreteValue::Range(range) = self {
            Some(range)
        } else {
            None
        }
    }

    pub fn display(&self) -> DustString {
        DustString::from(self.to_string())
    }

    pub fn r#type(&self) -> Type {
        match self {
            ConcreteValue::Boolean(_) => Type::Boolean,
            ConcreteValue::Byte(_) => Type::Byte,
            ConcreteValue::Character(_) => Type::Character,
            ConcreteValue::Float(_) => Type::Float,
            ConcreteValue::Integer(_) => Type::Integer,
            ConcreteValue::List(items) => items.first().map_or(Type::Any, |item| item.r#type()),
            ConcreteValue::Range(range) => range.r#type(),
            ConcreteValue::String(_) => Type::String,
            ConcreteValue::Function(chunk) => Type::Function(chunk.r#type.clone()),
        }
    }
}

impl Clone for ConcreteValue {
    fn clone(&self) -> Self {
        trace!("Cloning concrete value {}", self);

        match self {
            ConcreteValue::Boolean(boolean) => ConcreteValue::Boolean(*boolean),
            ConcreteValue::Byte(byte) => ConcreteValue::Byte(*byte),
            ConcreteValue::Character(character) => ConcreteValue::Character(*character),
            ConcreteValue::Float(float) => ConcreteValue::Float(*float),
            ConcreteValue::Integer(integer) => ConcreteValue::Integer(*integer),
            ConcreteValue::List(items) => ConcreteValue::List(items.clone()),
            ConcreteValue::Range(range) => ConcreteValue::Range(*range),
            ConcreteValue::String(string) => ConcreteValue::String(string.clone()),
            ConcreteValue::Function(chunk) => ConcreteValue::Function(chunk.clone()),
        }
    }
}

impl Display for ConcreteValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ConcreteValue::Boolean(boolean) => write!(f, "{boolean}"),
            ConcreteValue::Byte(byte) => write!(f, "0x{byte:02x}"),
            ConcreteValue::Character(character) => write!(f, "{character}"),
            ConcreteValue::Float(float) => {
                write!(f, "{float}")?;

                if float.fract() == 0.0 {
                    write!(f, ".0")?;
                }

                Ok(())
            }
            ConcreteValue::Integer(integer) => write!(f, "{integer}"),
            ConcreteValue::List(items) => {
                write!(f, "[")?;

                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{item}")?;
                }

                write!(f, "]")
            }
            ConcreteValue::Range(range_value) => {
                write!(f, "{range_value}")
            }
            ConcreteValue::String(string) => write!(f, "{string}"),
            ConcreteValue::Function(chunk) => write!(f, "{}", chunk.r#type),
        }
    }
}
