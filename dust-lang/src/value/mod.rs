mod dust_range;
mod list;

use std::{fmt::Display, sync::Arc};

use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};

pub use list::List;

use crate::{
    Type,
    chunk::{Chunk, StrippedChunk},
};

pub type DustString = SmartString<LazyCompact>;

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Value<C = StrippedChunk> {
    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Integer(i64),
    String(DustString),
    List(List<C>),
    Function(Arc<C>),
}

impl<C: Chunk> Value<C> {
    pub fn boolean(boolean: bool) -> Self {
        Value::Boolean(boolean)
    }

    pub fn byte(byte: u8) -> Self {
        Value::Byte(byte)
    }

    pub fn character(character: char) -> Self {
        Value::Character(character)
    }

    pub fn float(float: f64) -> Self {
        Value::Float(float)
    }

    pub fn integer(integer: i64) -> Self {
        Value::Integer(integer)
    }

    pub fn string<T: Into<DustString>>(value: T) -> Self {
        Value::String(value.into())
    }

    pub fn list<T: Into<List<C>>>(list: T) -> Self {
        Value::List(list.into())
    }

    pub fn function(function: Arc<C>) -> Self {
        Value::Function(function)
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
            Value::Byte(byte) => write!(f, "{byte}"),
            Value::Character(character) => write!(f, "{character}"),
            Value::Float(float) => write!(f, "{float}"),
            Value::Integer(integer) => write!(f, "{integer}"),
            Value::String(string) => write!(f, "{string}"),
            Value::List(list) => write!(f, "{list}"),
            Value::Function(function) => write!(f, "{}", function.r#type()),
        }
    }
}
