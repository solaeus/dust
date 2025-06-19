mod dust_range;
mod dust_string;
mod list;

use std::{cmp::Ordering, fmt::Display, sync::Arc};

use serde::{Deserialize, Serialize};

pub use dust_string::DustString;
pub use list::List;

use crate::{Chunk, StrippedChunk, Type};

#[derive(Clone, Debug, Serialize, Deserialize)]
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

    pub fn function(function: C) -> Self {
        Value::Function(Arc::new(function))
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

    pub fn as_boolean_or_panic(&self) -> bool {
        match self {
            Value::Boolean(boolean) => *boolean,
            _ => unreachable!("Attempted to use the value `{self}` as a boolean"),
        }
    }

    pub fn as_byte_or_panic(&self) -> u8 {
        match self {
            Value::Byte(byte) => *byte,
            _ => unreachable!("Attempted to use the value `{self}` as a byte"),
        }
    }

    pub fn as_character_or_panic(&self) -> char {
        match self {
            Value::Character(character) => *character,
            _ => unreachable!("Attempted to use the value `{self}` as a character"),
        }
    }

    pub fn as_float_or_panic(&self) -> f64 {
        match self {
            Value::Float(float) => *float,
            _ => unreachable!("Attempted to use the value `{self}` as a float"),
        }
    }

    pub fn as_integer_or_panic(&self) -> i64 {
        match self {
            Value::Integer(integer) => *integer,
            _ => unreachable!("Attempted to use the value `{self}` as an integer"),
        }
    }

    pub fn as_string_or_panic(&self) -> &DustString {
        match self {
            Value::String(string) => string,
            _ => unreachable!("Attempted to use the value `{self}` as a string"),
        }
    }

    pub fn as_boolean_list_or_panic(&self) -> &Vec<bool> {
        match self {
            Value::List(List::Boolean(booleans)) => booleans,
            _ => unreachable!("Attempted to use the value `{self}` as a boolean list"),
        }
    }

    pub fn as_byte_list_or_panic(&self) -> &Vec<u8> {
        match self {
            Value::List(List::Byte(bytes)) => bytes,
            _ => unreachable!("Attempted to use the value `{self}` as a byte list"),
        }
    }

    pub fn as_character_list_or_panic(&self) -> &Vec<char> {
        match self {
            Value::List(List::Character(characters)) => characters,
            _ => unreachable!("Attempted to use the value `{self}` as a character list"),
        }
    }

    pub fn as_float_list_or_panic(&self) -> &Vec<f64> {
        match self {
            Value::List(List::Float(floats)) => floats,
            _ => unreachable!("Attempted to use the value `{self}` as a float list"),
        }
    }

    pub fn as_integer_list_or_panic(&self) -> &Vec<i64> {
        match self {
            Value::List(List::Integer(integers)) => integers,
            _ => unreachable!("Attempted to use the value `{self}` as an integer list"),
        }
    }

    pub fn as_string_list_or_panic(&self) -> &Vec<DustString> {
        match self {
            Value::List(List::String(strings)) => strings,
            _ => unreachable!("Attempted to use the value `{self}` as a string list"),
        }
    }

    pub fn as_list_list_or_panic(&self) -> &Vec<List<C>> {
        match self {
            Value::List(List::List(lists)) => lists,
            _ => unreachable!("Attempted to use the value `{self}` as a list list"),
        }
    }

    pub fn as_function_list_or_panic(&self) -> &Vec<Arc<C>> {
        match self {
            Value::List(List::Function(functions)) => functions,
            _ => unreachable!("Attempted to use the value `{self}` as a function list"),
        }
    }

    pub fn as_function_or_panic(&self) -> &Arc<C> {
        match self {
            Value::Function(function) => function,
            _ => unreachable!("Attempted to use the value `{self}` as a function"),
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
            (Value::Function(left), Value::Function(right)) => Arc::ptr_eq(left, right),
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
            (Value::Function(left), Value::Function(right)) => {
                Arc::as_ptr(left).cmp(&Arc::as_ptr(right))
            }
            (Value::Function(_), _) => Ordering::Greater,
        }
    }
}
