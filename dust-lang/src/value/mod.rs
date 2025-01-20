//! Runtime values used by the VM.
mod abstract_list;
mod function;
mod range_value;

pub use abstract_list::AbstractList;
pub use function::Function;
pub use range_value::RangeValue;
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};

use std::fmt::{self, Debug, Display, Formatter};

use crate::{Type, vm::ThreadData};

pub type DustString = SmartString<LazyCompact>;

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Value {
    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Integer(i64),
    String(DustString),

    List(Vec<Value>),

    #[serde(skip)]
    AbstractList(AbstractList),
    #[serde(skip)]
    Function(Function),
}

impl Value {
    pub fn as_boolean(&self) -> Option<bool> {
        if let Value::Boolean(boolean) = self {
            Some(*boolean)
        } else {
            None
        }
    }

    pub fn as_byte(&self) -> Option<u8> {
        if let Value::Byte(byte) = self {
            Some(*byte)
        } else {
            None
        }
    }

    pub fn as_character(&self) -> Option<char> {
        if let Value::Character(character) = self {
            Some(*character)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        if let Value::Float(float) = self {
            Some(*float)
        } else {
            None
        }
    }

    pub fn as_function(&self) -> Option<&Function> {
        if let Value::Function(function) = self {
            Some(function)
        } else {
            None
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        if let Value::Integer(integer) = self {
            Some(*integer)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&DustString> {
        if let Value::String(string) = self {
            Some(string)
        } else {
            None
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Value::Function(_))
    }

    pub fn r#type(&self) -> Type {
        match self {
            Value::Boolean(_) => Type::Boolean,
            Value::Byte(_) => Type::Byte,
            Value::Character(_) => Type::Character,
            Value::Float(_) => Type::Float,
            Value::Integer(_) => Type::Integer,
            Value::String(_) => Type::String,
            Value::List(_) => Type::List(Box::new(Type::Any)),
            Value::AbstractList(AbstractList { item_type, .. }) => {
                Type::List(Box::new(item_type.clone()))
            }
            Value::Function(Function { r#type, .. }) => Type::Function(Box::new(r#type.clone())),
        }
    }

    pub fn display(&self, data: &ThreadData) -> DustString {
        match self {
            Value::Boolean(boolean) => DustString::from(boolean.to_string()),
            Value::Byte(byte) => DustString::from(byte.to_string()),
            Value::Character(character) => DustString::from(character.to_string()),
            Value::Float(float) => DustString::from(float.to_string()),
            Value::Integer(integer) => DustString::from(integer.to_string()),
            Value::List(list) => {
                let mut display = DustString::new();

                display.push_str("[");

                for (index, value) in list.iter().enumerate() {
                    if index > 0 {
                        display.push_str(", ");
                    }

                    display.push_str(&value.display(data));
                }

                display.push_str("]");

                display
            }
            Value::String(string) => string.clone(),
            Value::AbstractList(list) => list.display(data),
            Value::Function(function) => DustString::from(function.to_string()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Boolean(boolean) => write!(f, "{boolean}"),
            Value::Byte(byte) => write!(f, "{byte}"),
            Value::Character(character) => write!(f, "{character}"),
            Value::Float(float) => write!(f, "{float}"),
            Value::Integer(integer) => write!(f, "{integer}"),
            Value::String(string) => write!(f, "{string}"),
            Value::List(list) => {
                write!(f, "[")?;

                for (index, value) in list.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}", value)?;
                }

                write!(f, "]")
            }
            Value::AbstractList(list) => write!(f, "{list}"),
            Value::Function(function) => write!(f, "{function}"),
        }
    }
}
