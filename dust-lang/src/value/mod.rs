mod dust_range;
mod dust_string;
mod list;

use std::{
    cmp::Ordering,
    fmt::Display,
    hash::{Hash, Hasher},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

pub use dust_string::DustString;
pub use list::List;

use crate::{Chunk, StrippedChunk, Type};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Value<C = StrippedChunk> {
    Boolean(bool) = 0,
    Byte(u8) = 1,
    Character(char) = 2,
    Float(f64) = 3,
    Integer(i64) = 4,
    String(DustString) = 5,
    List(List<C>) = 6,
    Function(Arc<C>) = 7,
}

impl<C: Chunk> Value<C> {
    pub fn boolean(boolean: bool) -> Self {
        Value::Boolean(boolean)
    }

    pub fn as_boolean(&self) -> Option<bool> {
        if let Value::Boolean(boolean) = self {
            Some(*boolean)
        } else {
            None
        }
    }

    pub fn byte(byte: u8) -> Self {
        Value::Byte(byte)
    }

    pub fn as_byte(&self) -> Option<u8> {
        if let Value::Byte(byte) = self {
            Some(*byte)
        } else {
            None
        }
    }

    pub fn character(character: char) -> Self {
        Value::Character(character)
    }

    pub fn as_character(&self) -> Option<char> {
        if let Value::Character(character) = self {
            Some(*character)
        } else {
            None
        }
    }

    pub fn float(float: f64) -> Self {
        Value::Float(float)
    }

    pub fn as_float(&self) -> Option<f64> {
        if let Value::Float(float) = self {
            Some(*float)
        } else {
            None
        }
    }

    pub fn integer(integer: i64) -> Self {
        Value::Integer(integer)
    }

    pub fn as_integer(&self) -> Option<i64> {
        if let Value::Integer(integer) = self {
            Some(*integer)
        } else {
            None
        }
    }

    pub fn string<T: Into<DustString>>(value: T) -> Self {
        Value::String(value.into())
    }

    pub fn as_string(&self) -> Option<&DustString> {
        if let Value::String(string) = self {
            Some(string)
        } else {
            None
        }
    }

    pub fn boolean_list<T: Into<Vec<bool>>>(booleans: T) -> Self {
        Value::List(List::boolean(booleans))
    }

    pub fn byte_list<T: Into<Vec<u8>>>(bytes: T) -> Self {
        Value::List(List::byte(bytes))
    }

    pub fn character_list<T: Into<Vec<char>>>(characters: T) -> Self {
        Value::List(List::character(characters))
    }

    pub fn float_list<T: Into<Vec<f64>>>(floats: T) -> Self {
        Value::List(List::float(floats))
    }

    pub fn integer_list<T: Into<Vec<i64>>>(items: T) -> Self {
        Value::List(List::integer(items))
    }

    pub fn string_list<T: Into<Vec<DustString>>>(strings: T) -> Self {
        Value::List(List::string(strings))
    }

    pub fn list_list<T: Into<Vec<List<C>>>>(lists: T) -> Self {
        Value::List(List::list(lists))
    }

    pub fn function_list<T: Into<Vec<Arc<C>>>>(functions: T) -> Self {
        Value::List(List::function(functions))
    }

    pub fn as_list(&self) -> Option<&List<C>> {
        if let Value::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    pub fn into_list(self) -> Option<List<C>> {
        if let Value::List(list) = self {
            Some(list)
        } else {
            None
        }
    }

    pub fn function(chunk: C) -> Self {
        Value::Function(Arc::new(chunk))
    }

    pub fn as_function(&self) -> Option<&Arc<C>> {
        if let Value::Function(function) = self {
            Some(function)
        } else {
            None
        }
    }

    pub fn r#type(&self) -> Type {
        match self {
            Value::Boolean(_) => Type::Boolean,
            Value::Byte(_) => Type::Byte,
            Value::Character(_) => Type::Character,
            Value::Float(_) => Type::Float,
            Value::Integer(_) => Type::Integer,
            Value::String(_) => Type::String,
            Value::List(list) => list.r#type(),
            Value::Function(function) => Type::Function(Box::new(function.r#type().clone())),
        }
    }
}

impl<C: Chunk> Display for Value<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Boolean(boolean) => write!(f, "{boolean}"),
            Value::Byte(byte) => write!(f, "{byte:#04X}"),
            Value::Character(character) => write!(f, "{character}"),
            Value::Float(float) => write!(f, "{float}"),
            Value::Integer(integer) => write!(f, "{integer}"),
            Value::String(string) => write!(f, "{string}"),
            Value::List(list) => write!(f, "{list}"),
            Value::Function(function) => write!(f, "{}", function.r#type()),
        }
    }
}

impl<C: PartialEq> Eq for Value<C> {}

impl<C: PartialEq> PartialEq for Value<C> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Boolean(left), Value::Boolean(right)) => left == right,
            (Value::Byte(left), Value::Byte(right)) => left == right,
            (Value::Character(left), Value::Character(right)) => left == right,
            (Value::Float(left), Value::Float(right)) => left.to_bits() == right.to_bits(),
            (Value::Integer(left), Value::Integer(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::List(left), Value::List(right)) => left == right,
            (Value::Function(left), Value::Function(right)) => left == right,
            _ => false,
        }
    }
}

impl<C: Ord> PartialOrd for Value<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<C: Ord> Ord for Value<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Value::Boolean(left), Value::Boolean(right)) => left.cmp(right),
            (Value::Boolean(_), _) => Ordering::Less,
            (Value::Byte(left), Value::Byte(right)) => left.cmp(right),
            (Value::Byte(_), _) => Ordering::Less,
            (Value::Character(left), Value::Character(right)) => left.cmp(right),
            (Value::Character(_), _) => Ordering::Less,
            (Value::Float(left), Value::Float(right)) => left.total_cmp(right),
            (Value::Float(_), _) => Ordering::Less,
            (Value::Integer(left), Value::Integer(right)) => left.cmp(right),
            (Value::Integer(_), _) => Ordering::Less,
            (Value::String(left), Value::String(right)) => left.cmp(right),
            (Value::String(_), _) => Ordering::Less,
            (Value::List(left), Value::List(right)) => left.cmp(right),
            (Value::List(_), _) => Ordering::Less,
            (Value::Function(left), Value::Function(right)) => left.cmp(right),
            (Value::Function(_), _) => Ordering::Greater,
        }
    }
}

impl Hash for Value<StrippedChunk> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Boolean(value) => value.hash(state),
            Value::Byte(value) => value.hash(state),
            Value::Character(value) => value.hash(state),
            Value::Float(value) => value.to_bits().hash(state),
            Value::Integer(value) => value.hash(state),
            Value::String(value) => value.hash(state),
            Value::List(value) => value.hash(state),
            Value::Function(value) => Arc::as_ptr(value).hash(state),
        }
    }
}
