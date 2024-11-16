//! Runtime values used by the VM.
mod range;

pub use range::RangeValue;

use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Chunk, Type};

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Value {
    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Function(Chunk),
    Integer(i64),
    List(Vec<Value>),
    Range(RangeValue),
    String(String),
}

impl Value {
    pub fn boolean(value: bool) -> Self {
        Value::Boolean(value)
    }

    pub fn byte(value: u8) -> Self {
        Value::Byte(value)
    }

    pub fn character(value: char) -> Self {
        Value::Character(value)
    }

    pub fn float(value: f64) -> Self {
        Value::Float(value)
    }

    pub fn function(chunk: Chunk) -> Self {
        Value::Function(chunk)
    }

    pub fn integer<T: Into<i64>>(into_i64: T) -> Self {
        Value::Integer(into_i64.into())
    }

    pub fn list<T: Into<Vec<Value>>>(into_list: T) -> Self {
        Value::List(into_list.into())
    }

    pub fn range(range: RangeValue) -> Self {
        Value::Range(range)
    }

    pub fn string<T: ToString>(to_string: T) -> Self {
        Value::String(to_string.to_string())
    }

    pub fn as_string(&self) -> Option<&String> {
        if let Value::String(string) = self {
            Some(string)
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
            Value::Function(chunk) => Type::Function(chunk.r#type().clone()),
            Value::Integer(_) => Type::Integer,
            Value::List(list) => {
                let item_type = list.first().map_or(Type::Any, |item| item.r#type());

                Type::List {
                    item_type: Box::new(item_type),
                    length: list.len(),
                }
            }
            Value::Range(range) => range.r#type(),
            Value::String(string) => Type::String {
                length: Some(string.len()),
            },
        }
    }

    pub fn add(&self, other: &Self) -> Result<Value, ValueError> {
        use Value::*;

        let sum = match (self, other) {
            (Byte(left), Byte(right)) => Value::byte(left.saturating_add(*right)),
            (Float(left), Float(right)) => Value::float(*left + *right),
            (Integer(left), Integer(right)) => Value::integer(left.saturating_add(*right)),
            (String(left), String(right)) => Value::string(format!("{}{}", left, right)),
            _ => return Err(ValueError::CannotAdd(self.clone(), other.clone())),
        };

        Ok(sum)
    }

    pub fn subtract(&self, other: &Self) -> Result<Value, ValueError> {
        use Value::*;

        let difference = match (self, other) {
            (Byte(left), Byte(right)) => Value::byte(left.saturating_sub(*right)),
            (Float(left), Float(right)) => Value::float(left - right),
            (Integer(left), Integer(right)) => Value::integer(left.saturating_sub(*right)),
            _ => return Err(ValueError::CannotSubtract(self.clone(), other.clone())),
        };

        Ok(difference)
    }

    pub fn multiply(&self, other: &Self) -> Result<Value, ValueError> {
        use Value::*;

        let product = match (self, other) {
            (Byte(left), Byte(right)) => Value::byte(left.saturating_mul(*right)),
            (Float(left), Float(right)) => Value::float(left * right),
            (Integer(left), Integer(right)) => Value::integer(left.saturating_mul(*right)),
            _ => return Err(ValueError::CannotMultiply(self.clone(), other.clone())),
        };

        Ok(product)
    }

    pub fn divide(&self, other: &Self) -> Result<Value, ValueError> {
        use Value::*;

        let quotient = match (self, other) {
            (Byte(left), Byte(right)) => Value::byte(left.saturating_div(*right)),
            (Float(left), Float(right)) => Value::float(left / right),
            (Integer(left), Integer(right)) => Value::integer(left.saturating_div(*right)),
            _ => return Err(ValueError::CannotMultiply(self.clone(), other.clone())),
        };

        Ok(quotient)
    }

    pub fn modulo(&self, other: &Self) -> Result<Value, ValueError> {
        use Value::*;

        let product = match (self, other) {
            (Byte(left), Byte(right)) => Value::byte(left.wrapping_rem(*right)),
            (Float(left), Float(right)) => Value::float(left % right),
            (Integer(left), Integer(right)) => Value::integer(left.wrapping_rem_euclid(*right)),
            _ => return Err(ValueError::CannotMultiply(self.clone(), other.clone())),
        };

        Ok(product)
    }

    pub fn negate(&self) -> Result<Value, ValueError> {
        use Value::*;

        let negated = match self {
            Boolean(value) => Value::boolean(!value),
            Byte(value) => Value::byte(value.wrapping_neg()),
            Float(value) => Value::float(-value),
            Integer(value) => Value::integer(value.wrapping_neg()),
            _ => return Err(ValueError::CannotNegate(self.clone())),
        };

        Ok(negated)
    }

    pub fn not(&self) -> Result<Value, ValueError> {
        use Value::*;

        let not = match self {
            Boolean(value) => Value::boolean(!value),
            _ => return Err(ValueError::CannotNot(self.clone())),
        };

        Ok(not)
    }

    pub fn equal(&self, other: &Value) -> Result<Value, ValueError> {
        use Value::*;

        let equal = match (self, other) {
            (Boolean(left), Boolean(right)) => Value::boolean(left == right),
            (Byte(left), Byte(right)) => Value::boolean(left == right),
            (Character(left), Character(right)) => Value::boolean(left == right),
            (Float(left), Float(right)) => Value::boolean(left == right),
            (Function(left), Function(right)) => Value::boolean(left == right),
            (Integer(left), Integer(right)) => Value::boolean(left == right),
            (List(left), List(right)) => Value::boolean(left == right),
            (Range(left), Range(right)) => Value::boolean(left == right),
            (String(left), String(right)) => Value::boolean(left == right),
            _ => return Err(ValueError::CannotCompare(self.clone(), other.clone())),
        };

        Ok(equal)
    }

    pub fn less_than(&self, other: &Value) -> Result<Value, ValueError> {
        use Value::*;

        let less_than = match (self, other) {
            (Boolean(left), Boolean(right)) => Value::boolean(left < right),
            (Byte(left), Byte(right)) => Value::boolean(left < right),
            (Character(left), Character(right)) => Value::boolean(left < right),
            (Float(left), Float(right)) => Value::boolean(left < right),
            (Function(left), Function(right)) => Value::boolean(left < right),
            (Integer(left), Integer(right)) => Value::boolean(left < right),
            (List(left), List(right)) => Value::boolean(left < right),
            (Range(left), Range(right)) => Value::boolean(left < right),
            (String(left), String(right)) => Value::boolean(left < right),
            _ => return Err(ValueError::CannotCompare(self.clone(), other.clone())),
        };

        Ok(less_than)
    }

    pub fn less_than_or_equal(&self, other: &Value) -> Result<Value, ValueError> {
        use Value::*;

        let less_than_or_equal = match (self, other) {
            (Boolean(left), Boolean(right)) => Value::boolean(left <= right),
            (Byte(left), Byte(right)) => Value::boolean(left <= right),
            (Character(left), Character(right)) => Value::boolean(left <= right),
            (Float(left), Float(right)) => Value::boolean(left <= right),
            (Function(left), Function(right)) => Value::boolean(left <= right),
            (Integer(left), Integer(right)) => Value::boolean(left <= right),
            (List(left), List(right)) => Value::boolean(left <= right),
            (Range(left), Range(right)) => Value::boolean(left <= right),
            (String(left), String(right)) => Value::boolean(left <= right),
            _ => return Err(ValueError::CannotCompare(self.clone(), other.clone())),
        };

        Ok(less_than_or_equal)
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        log::trace!("Cloning concrete value {}", self);

        match self {
            Value::Boolean(boolean) => Value::Boolean(*boolean),
            Value::Byte(byte) => Value::Byte(*byte),
            Value::Character(character) => Value::Character(*character),
            Value::Float(float) => Value::Float(*float),
            Value::Function(function) => Value::Function(function.clone()),
            Value::Integer(integer) => Value::Integer(*integer),
            Value::List(list) => Value::List(list.clone()),
            Value::Range(range) => Value::Range(*range),
            Value::String(string) => Value::String(string.clone()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Boolean(boolean) => write!(f, "{boolean}"),
            Value::Byte(byte) => write!(f, "0x{byte:02x}"),
            Value::Character(character) => write!(f, "{character}"),
            Value::Float(float) => {
                write!(f, "{float}")?;

                if float.fract() == 0.0 {
                    write!(f, ".0")?;
                }

                Ok(())
            }
            Value::Function(function) => write!(f, "{function}"),
            Value::Integer(integer) => write!(f, "{integer}"),
            Value::List(list) => {
                write!(f, "[")?;

                for (index, element) in list.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{element}")?;
                }

                write!(f, "]")
            }
            Value::Range(range_value) => {
                write!(f, "{range_value}")
            }
            Value::String(string) => write!(f, "{string}"),
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
