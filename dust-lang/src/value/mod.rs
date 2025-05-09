//! Runtime values used by the VM.
mod abstract_list;
mod concrete_value;
mod function;
mod range_value;

pub use abstract_list::AbstractList;
pub use concrete_value::{ConcreteValue, DustString};
pub use function::Function;
pub use range_value::RangeValue;
use serde::{Deserialize, Serialize};
use tracing::warn;

use std::fmt::{self, Debug, Display, Formatter};

use crate::Type;

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Value {
    Concrete(ConcreteValue),

    #[serde(skip)]
    AbstractList(AbstractList),

    #[serde(skip)]
    Function(Function),
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

    pub fn as_function(&self) -> Option<&Function> {
        if let Value::Function(function) = self {
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
        matches!(self, Value::Function(_))
    }

    pub fn r#type(&self) -> Type {
        match self {
            Value::Concrete(concrete_value) => concrete_value.r#type(),
            Value::AbstractList(AbstractList { item_type, .. }) => Type::List(*item_type),
            Value::Function(Function { r#type, .. }) => Type::Function(r#type.clone()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Concrete(concrete_value) => write!(f, "{concrete_value}"),
            Value::AbstractList(list) => {
                warn!(
                    "Using Display implementation on an AbstractList. Use AbstractList::display instead."
                );
                write!(f, "{list:?}")
            }
            Value::Function(function) => {
                warn!("Using Display implementation on a Function. Use Function::display instead.");
                write!(f, "{function:?}")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueError {
    CannotAdd(Value, Value),
    CannotAnd(Value, Value),
    CannotCompare(Value, Value),
    CannotDivide(Value, Value),
    CannotModulo(Value, Value),
    CannotMultiply(Value, Value),
    CannotNegate(Value),
    CannotNot(Value),
    CannotSubtract(Value, Value),
    CannotOr(Value, Value),
}

impl Display for ValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ValueError::CannotAdd(left, right) => {
                write!(f, "Cannot add {left} and {right}")
            }
            ValueError::CannotAnd(left, right) => {
                write!(f, "Cannot use logical AND operation on {left} and {right}")
            }
            ValueError::CannotCompare(left, right) => {
                write!(f, "Cannot compare {left} and {right}")
            }
            ValueError::CannotDivide(left, right) => {
                write!(f, "Cannot divide {left} by {right}")
            }
            ValueError::CannotModulo(left, right) => {
                write!(f, "Cannot use modulo operation on {left} and {right}")
            }
            ValueError::CannotMultiply(left, right) => {
                write!(f, "Cannot multiply {left} by {right}")
            }
            ValueError::CannotNegate(value) => {
                write!(f, "Cannot negate {value}")
            }
            ValueError::CannotNot(value) => {
                write!(f, "Cannot use logical NOT operation on {value}")
            }
            ValueError::CannotSubtract(left, right) => {
                write!(f, "Cannot subtract {right} from {left}")
            }
            ValueError::CannotOr(left, right) => {
                write!(f, "Cannot use logical OR operation on {left} and {right}")
            }
        }
    }
}
