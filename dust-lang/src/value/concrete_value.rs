use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use tracing::trace;

use crate::{Type, Value, ValueError};

use super::RangeValue;

pub type DustString = SmartString<LazyCompact>;

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ConcreteValue {
    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Integer(i64),
    List(Vec<ConcreteValue>),
    Range(RangeValue),
    String(DustString),
}

impl ConcreteValue {
    pub fn to_value(self) -> Value {
        Value::Concrete(self)
    }

    pub fn list<T: Into<Vec<ConcreteValue>>>(into_list: T) -> Self {
        ConcreteValue::List(into_list.into())
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
            ConcreteValue::List(list) => {
                let item_type = list
                    .first()
                    .map_or(Type::Any, |item| item.r#type())
                    .type_code();

                Type::List(item_type)
            }
            ConcreteValue::Range(range) => range.r#type(),
            ConcreteValue::String(_) => Type::String,
        }
    }

    pub fn add(&self, other: &Self) -> ConcreteValue {
        use ConcreteValue::*;

        match (self, other) {
            (Byte(left), Byte(right)) => {
                let sum = left.saturating_add(*right);

                Byte(sum)
            }
            (Character(left), Character(right)) => {
                let mut concatenated = DustString::new();

                concatenated.push(*left);
                concatenated.push(*right);

                String(concatenated)
            }
            (Character(left), String(right)) => {
                let mut concatenated = DustString::new();

                concatenated.push(*left);
                concatenated.push_str(right);

                String(concatenated)
            }
            (Float(left), Float(right)) => {
                let sum = left + right;

                Float(sum)
            }
            (Integer(left), Integer(right)) => {
                let sum = left.saturating_add(*right);

                Integer(sum)
            }
            (String(left), Character(right)) => {
                let concatenated = format!("{}{}", left, right);

                String(DustString::from(concatenated))
            }
            (String(left), String(right)) => {
                let concatenated = format!("{}{}", left, right);

                String(DustString::from(concatenated))
            }
            _ => panic!(
                "{}",
                ValueError::CannotAdd(
                    Value::Concrete(self.clone()),
                    Value::Concrete(other.clone())
                )
            ),
        }
    }

    pub fn subtract(&self, other: &Self) -> ConcreteValue {
        use ConcreteValue::*;

        match (self, other) {
            (Byte(left), Byte(right)) => ConcreteValue::Byte(left.saturating_sub(*right)),
            (Float(left), Float(right)) => ConcreteValue::Float(left - right),
            (Integer(left), Integer(right)) => ConcreteValue::Integer(left.saturating_sub(*right)),
            _ => panic!(
                "{}",
                ValueError::CannotSubtract(
                    Value::Concrete(self.clone()),
                    Value::Concrete(other.clone())
                )
            ),
        }
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
                ));
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
                ));
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
                ));
            }
        };

        Ok(product)
    }

    pub fn negate(&self) -> ConcreteValue {
        use ConcreteValue::*;

        match self {
            Boolean(value) => ConcreteValue::Boolean(!value),
            Byte(value) => ConcreteValue::Byte(value.wrapping_neg()),
            Float(value) => ConcreteValue::Float(-value),
            Integer(value) => ConcreteValue::Integer(value.wrapping_neg()),
            _ => panic!("{}", ValueError::CannotNegate(self.clone().to_value())),
        }
    }

    pub fn not(&self) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let not = match self {
            Boolean(value) => ConcreteValue::Boolean(!value),
            _ => return Err(ValueError::CannotNot(self.clone().to_value())),
        };

        Ok(not)
    }

    pub fn equals(&self, other: &ConcreteValue) -> bool {
        use ConcreteValue::*;

        match (self, other) {
            (Boolean(left), Boolean(right)) => left == right,
            (Byte(left), Byte(right)) => left == right,
            (Character(left), Character(right)) => left == right,
            (Float(left), Float(right)) => left == right,
            (Integer(left), Integer(right)) => left == right,
            (List(left), List(right)) => left == right,
            (Range(left), Range(right)) => left == right,
            (String(left), String(right)) => left == right,
            _ => {
                panic!(
                    "{}",
                    ValueError::CannotCompare(
                        Value::Concrete(self.clone()),
                        Value::Concrete(other.clone())
                    )
                )
            }
        }
    }

    pub fn less_than(&self, other: &ConcreteValue) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let less_than = match (self, other) {
            (Boolean(left), Boolean(right)) => ConcreteValue::Boolean(left < right),
            (Byte(left), Byte(right)) => ConcreteValue::Boolean(left < right),
            (Character(left), Character(right)) => ConcreteValue::Boolean(left < right),
            (Float(left), Float(right)) => ConcreteValue::Boolean(left < right),
            (Integer(left), Integer(right)) => ConcreteValue::Boolean(left < right),
            (List(left), List(right)) => ConcreteValue::Boolean(left < right),
            (Range(left), Range(right)) => ConcreteValue::Boolean(left < right),
            (String(left), String(right)) => ConcreteValue::Boolean(left < right),
            _ => {
                return Err(ValueError::CannotCompare(
                    self.clone().to_value(),
                    other.clone().to_value(),
                ));
            }
        };

        Ok(less_than)
    }

    pub fn less_than_or_equals(&self, other: &ConcreteValue) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let less_than_or_equal = match (self, other) {
            (Boolean(left), Boolean(right)) => ConcreteValue::Boolean(left <= right),
            (Byte(left), Byte(right)) => ConcreteValue::Boolean(left <= right),
            (Character(left), Character(right)) => ConcreteValue::Boolean(left <= right),
            (Float(left), Float(right)) => ConcreteValue::Boolean(left <= right),
            (Integer(left), Integer(right)) => ConcreteValue::Boolean(left <= right),
            (List(left), List(right)) => ConcreteValue::Boolean(left <= right),
            (Range(left), Range(right)) => ConcreteValue::Boolean(left <= right),
            (String(left), String(right)) => ConcreteValue::Boolean(left <= right),
            _ => {
                return Err(ValueError::CannotCompare(
                    self.clone().to_value(),
                    other.clone().to_value(),
                ));
            }
        };

        Ok(less_than_or_equal)
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
