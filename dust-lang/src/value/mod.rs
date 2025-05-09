//! Runtime values used by the VM.
mod abstract_value;
mod concrete_value;

pub use abstract_value::{AbstractFunction, AbstractList, AbstractValue};
use concrete_value::ConcreteFunction;
pub use concrete_value::ConcreteValue;
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};

use std::fmt::{self, Debug, Display, Formatter};

use crate::Type;

pub type DustString = SmartString<LazyCompact>;

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Value {
    #[serde(skip)]
    Abstract(AbstractValue),
    Concrete(ConcreteValue),
}

impl Value {
    pub fn boolean(boolean: bool) -> Self {
        Value::Concrete(ConcreteValue::Boolean(boolean))
    }

    pub fn byte(byte: u8) -> Self {
        Value::Concrete(ConcreteValue::Byte(byte))
    }

    pub fn character(character: char) -> Self {
        Value::Concrete(ConcreteValue::Character(character))
    }

    pub fn float(float: f64) -> Self {
        Value::Concrete(ConcreteValue::Float(float))
    }

    pub fn integer(integer: i64) -> Self {
        Value::Concrete(ConcreteValue::Integer(integer))
    }

    pub fn string(string: impl Into<DustString>) -> Self {
        Value::Concrete(ConcreteValue::String(string.into()))
    }

    pub fn as_concrete(&self) -> Option<&ConcreteValue> {
        if let Value::Concrete(concrete_value) = self {
            Some(concrete_value)
        } else {
            None
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        if let Value::Concrete(ConcreteValue::Boolean(boolean)) = self {
            Some(*boolean)
        } else {
            None
        }
    }

    pub fn as_byte(&self) -> Option<u8> {
        if let Value::Concrete(ConcreteValue::Byte(byte)) = self {
            Some(*byte)
        } else {
            None
        }
    }

    pub fn as_character(&self) -> Option<char> {
        if let Value::Concrete(ConcreteValue::Character(character)) = self {
            Some(*character)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        if let Value::Concrete(ConcreteValue::Float(float)) = self {
            Some(*float)
        } else {
            None
        }
    }

    pub fn as_concrete_function(&self) -> Option<&ConcreteFunction> {
        if let Value::Concrete(ConcreteValue::Function(function)) = self {
            Some(function)
        } else {
            None
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        if let Value::Concrete(ConcreteValue::Integer(integer)) = self {
            Some(*integer)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&DustString> {
        if let Value::Concrete(ConcreteValue::String(value)) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::Concrete(ConcreteValue::String(_)))
    }

    pub fn is_function(&self) -> bool {
        matches!(
            self,
            Value::Concrete(ConcreteValue::Function(_))
                | Value::Abstract(AbstractValue::Function(_))
        )
    }
}

/// An ordered sequence of values. These variants mirror the range types in `std::range`. This type
/// is not used on its own but forms the basis for Dust's [`ConcreteRange`] values.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum DustRange<T> {
    FromStart { start: T },
    Full,
    Inclusive { start: T, end: T },
    SemiInclusive { start: T, end: T },
    ToEnd { end: T },
    ToEndInclusive { end: T },
}

impl DustRange<u8> {
    pub fn r#type(&self) -> Type {
        Type::Range(Box::new(Type::Byte))
    }
}

impl DustRange<char> {
    pub fn r#type(&self) -> Type {
        Type::Range(Box::new(Type::Character))
    }
}

impl DustRange<f64> {
    pub fn r#type(&self) -> Type {
        Type::Range(Box::new(Type::Float))
    }
}

impl DustRange<i64> {
    pub fn r#type(&self) -> Type {
        Type::Range(Box::new(Type::Integer))
    }
}

impl<T: Display> Display for DustRange<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DustRange::FromStart { start } => write!(f, "{start}.."),
            DustRange::Full => write!(f, ".."),
            DustRange::Inclusive { start, end } => write!(f, "{start}..={end}"),
            DustRange::SemiInclusive { start, end } => write!(f, "{start}..{end}"),
            DustRange::ToEnd { end } => write!(f, "..{end}"),
            DustRange::ToEndInclusive { end } => write!(f, "..={end}"),
        }
    }
}
