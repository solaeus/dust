//! Runtime values used by the VM.
mod function;
mod range_value;

pub use function::Function;
pub use range_value::RangeValue;
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};

use std::fmt::{self, Debug, Display, Formatter};

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
    Function(Function),
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
            Value::List(list) => write!(f, "{list:?}"),
            Value::Function(function) => write!(f, "{function}"),
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
