use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Chunk, Type, Value, ValueError, ValueRef};

use super::RangeValue;

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ConcreteValue {
    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Function(Chunk),
    Integer(i64),
    List(Vec<ConcreteValue>),
    Range(RangeValue),
    String(String),
}

impl ConcreteValue {
    pub fn to_value(self) -> Value {
        Value::Concrete(self)
    }

    pub fn to_value_ref(&self) -> ValueRef {
        ValueRef::Concrete(self)
    }

    pub fn list<T: Into<Vec<ConcreteValue>>>(into_list: T) -> Self {
        ConcreteValue::List(into_list.into())
    }

    pub fn string<T: ToString>(to_string: T) -> Self {
        ConcreteValue::String(to_string.to_string())
    }

    pub fn as_string(&self) -> Option<&String> {
        if let ConcreteValue::String(string) = self {
            Some(string)
        } else {
            None
        }
    }

    pub fn r#type(&self) -> Type {
        match self {
            ConcreteValue::Boolean(_) => Type::Boolean,
            ConcreteValue::Byte(_) => Type::Byte,
            ConcreteValue::Character(_) => Type::Character,
            ConcreteValue::Float(_) => Type::Float,
            ConcreteValue::Function(chunk) => Type::Function(chunk.r#type().clone()),
            ConcreteValue::Integer(_) => Type::Integer,
            ConcreteValue::List(list) => {
                let item_type = list.first().map_or(Type::Any, |item| item.r#type());

                Type::List(Box::new(item_type))
            }
            ConcreteValue::Range(range) => range.r#type(),
            ConcreteValue::String(_) => Type::String,
        }
    }

    pub fn add(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let sum = match (self, other) {
            (Byte(left), Byte(right)) => ConcreteValue::Byte(left.saturating_add(*right)),
            (Float(left), Float(right)) => ConcreteValue::Float(*left + *right),
            (Integer(left), Integer(right)) => ConcreteValue::Integer(left.saturating_add(*right)),
            (String(left), String(right)) => ConcreteValue::string(format!("{}{}", left, right)),
            _ => {
                return Err(ValueError::CannotAdd(
                    self.clone().to_value(),
                    other.clone().to_value(),
                ))
            }
        };

        Ok(sum)
    }

    pub fn subtract(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let difference = match (self, other) {
            (Byte(left), Byte(right)) => ConcreteValue::Byte(left.saturating_sub(*right)),
            (Float(left), Float(right)) => ConcreteValue::Float(left - right),
            (Integer(left), Integer(right)) => ConcreteValue::Integer(left.saturating_sub(*right)),
            _ => {
                return Err(ValueError::CannotSubtract(
                    self.clone().to_value(),
                    other.clone().to_value(),
                ))
            }
        };

        Ok(difference)
    }

    pub fn multiply(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let product = match (self, other) {
            (Byte(left), Byte(right)) => ConcreteValue::Byte(left.saturating_mul(*right)),
            (Float(left), Float(right)) => ConcreteValue::Float(left * right),
            (Integer(left), Integer(right)) => ConcreteValue::Integer(left.saturating_mul(*right)),
            _ => {
                return Err(ValueError::CannotMultiply(
                    self.clone().to_value(),
                    other.clone().to_value(),
                ))
            }
        };

        Ok(product)
    }

    pub fn divide(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let quotient = match (self, other) {
            (Byte(left), Byte(right)) => ConcreteValue::Byte(left.saturating_div(*right)),
            (Float(left), Float(right)) => ConcreteValue::Float(left / right),
            (Integer(left), Integer(right)) => ConcreteValue::Integer(left.saturating_div(*right)),
            _ => {
                return Err(ValueError::CannotMultiply(
                    self.clone().to_value(),
                    other.clone().to_value(),
                ))
            }
        };

        Ok(quotient)
    }

    pub fn modulo(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let product = match (self, other) {
            (Byte(left), Byte(right)) => ConcreteValue::Byte(left.wrapping_rem(*right)),
            (Float(left), Float(right)) => ConcreteValue::Float(left % right),
            (Integer(left), Integer(right)) => {
                ConcreteValue::Integer(left.wrapping_rem_euclid(*right))
            }
            _ => {
                return Err(ValueError::CannotMultiply(
                    self.clone().to_value(),
                    other.clone().to_value(),
                ))
            }
        };

        Ok(product)
    }

    pub fn negate(&self) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let negated = match self {
            Boolean(value) => ConcreteValue::Boolean(!value),
            Byte(value) => ConcreteValue::Byte(value.wrapping_neg()),
            Float(value) => ConcreteValue::Float(-value),
            Integer(value) => ConcreteValue::Integer(value.wrapping_neg()),
            _ => return Err(ValueError::CannotNegate(self.clone().to_value())),
        };

        Ok(negated)
    }

    pub fn not(&self) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let not = match self {
            Boolean(value) => ConcreteValue::Boolean(!value),
            _ => return Err(ValueError::CannotNot(self.clone().to_value())),
        };

        Ok(not)
    }

    pub fn equal(&self, other: &ConcreteValue) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let equal = match (self, other) {
            (Boolean(left), Boolean(right)) => ConcreteValue::Boolean(left == right),
            (Byte(left), Byte(right)) => ConcreteValue::Boolean(left == right),
            (Character(left), Character(right)) => ConcreteValue::Boolean(left == right),
            (Float(left), Float(right)) => ConcreteValue::Boolean(left == right),
            (Function(left), Function(right)) => ConcreteValue::Boolean(left == right),
            (Integer(left), Integer(right)) => ConcreteValue::Boolean(left == right),
            (List(left), List(right)) => ConcreteValue::Boolean(left == right),
            (Range(left), Range(right)) => ConcreteValue::Boolean(left == right),
            (String(left), String(right)) => ConcreteValue::Boolean(left == right),
            _ => {
                return Err(ValueError::CannotCompare(
                    self.clone().to_value(),
                    other.clone().to_value(),
                ))
            }
        };

        Ok(equal)
    }

    pub fn less_than(&self, other: &ConcreteValue) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let less_than = match (self, other) {
            (Boolean(left), Boolean(right)) => ConcreteValue::Boolean(left < right),
            (Byte(left), Byte(right)) => ConcreteValue::Boolean(left < right),
            (Character(left), Character(right)) => ConcreteValue::Boolean(left < right),
            (Float(left), Float(right)) => ConcreteValue::Boolean(left < right),
            (Function(left), Function(right)) => ConcreteValue::Boolean(left < right),
            (Integer(left), Integer(right)) => ConcreteValue::Boolean(left < right),
            (List(left), List(right)) => ConcreteValue::Boolean(left < right),
            (Range(left), Range(right)) => ConcreteValue::Boolean(left < right),
            (String(left), String(right)) => ConcreteValue::Boolean(left < right),
            _ => {
                return Err(ValueError::CannotCompare(
                    self.clone().to_value(),
                    other.clone().to_value(),
                ))
            }
        };

        Ok(less_than)
    }

    pub fn less_than_or_equal(&self, other: &ConcreteValue) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let less_than_or_equal = match (self, other) {
            (Boolean(left), Boolean(right)) => ConcreteValue::Boolean(left <= right),
            (Byte(left), Byte(right)) => ConcreteValue::Boolean(left <= right),
            (Character(left), Character(right)) => ConcreteValue::Boolean(left <= right),
            (Float(left), Float(right)) => ConcreteValue::Boolean(left <= right),
            (Function(left), Function(right)) => ConcreteValue::Boolean(left <= right),
            (Integer(left), Integer(right)) => ConcreteValue::Boolean(left <= right),
            (List(left), List(right)) => ConcreteValue::Boolean(left <= right),
            (Range(left), Range(right)) => ConcreteValue::Boolean(left <= right),
            (String(left), String(right)) => ConcreteValue::Boolean(left <= right),
            _ => {
                return Err(ValueError::CannotCompare(
                    self.clone().to_value(),
                    other.clone().to_value(),
                ))
            }
        };

        Ok(less_than_or_equal)
    }
}

impl Clone for ConcreteValue {
    fn clone(&self) -> Self {
        log::trace!("Cloning concrete value {}", self);

        match self {
            ConcreteValue::Boolean(boolean) => ConcreteValue::Boolean(*boolean),
            ConcreteValue::Byte(byte) => ConcreteValue::Byte(*byte),
            ConcreteValue::Character(character) => ConcreteValue::Character(*character),
            ConcreteValue::Float(float) => ConcreteValue::Float(*float),
            ConcreteValue::Function(function) => ConcreteValue::Function(function.clone()),
            ConcreteValue::Integer(integer) => ConcreteValue::Integer(*integer),
            ConcreteValue::List(list) => ConcreteValue::List(list.clone()),
            ConcreteValue::Range(range) => ConcreteValue::Range(*range),
            ConcreteValue::String(string) => ConcreteValue::String(string.clone()),
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
            ConcreteValue::Function(chunk) => write!(f, "{}", chunk.r#type()),
            ConcreteValue::Integer(integer) => write!(f, "{integer}"),
            ConcreteValue::List(list) => {
                write!(f, "[")?;

                for (index, item) in list.iter().enumerate() {
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
        }
    }
}
