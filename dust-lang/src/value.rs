//! Runtime values used by the VM.
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter},
    ops::{Range, RangeInclusive},
};

use serde::{Deserialize, Serialize};

use crate::{function::Function, Chunk, FunctionBorrowed, Type};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReferenceValue<'a> {
    Concrete(&'a ConcreteValue),
    Function(FunctionBorrowed<'a>),
    List(&'a Vec<&'a ConcreteValue>),
}

impl<'a> ReferenceValue<'a> {
    pub fn to_value(self) -> Value<'a> {
        Value::Reference(self)
    }

    pub fn r#type(&self) -> Type {
        match self {
            ReferenceValue::Concrete(concrete) => concrete.r#type(),
            ReferenceValue::Function(function) => function.r#type(),
            ReferenceValue::List(list) => {
                let item_type = list
                    .first()
                    .map(|value| value.r#type())
                    .unwrap_or(Type::Any);

                Type::List {
                    item_type: Box::new(item_type),
                    length: list.len(),
                }
            }
        }
    }

    pub fn into_concrete(self) -> ConcreteValue {
        match self {
            ReferenceValue::Concrete(concrete) => concrete.clone(),
            ReferenceValue::Function(function) => {
                ConcreteValue::Function(Function::new(function.chunk().clone()))
            }
            ReferenceValue::List(list) => {
                let mut items = Vec::new();

                for value in list {
                    items.push((*value).clone());
                }

                ConcreteValue::List(items)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value<'a> {
    Concrete(ConcreteValue),
    Reference(ReferenceValue<'a>),
}

impl<'a> Value<'a> {
    pub fn r#type(&self) -> Type {
        match self {
            Value::Concrete(concrete) => concrete.r#type(),
            Value::Reference(reference) => reference.r#type(),
        }
    }

    pub fn into_concrete(self) -> ConcreteValue {
        match self {
            Value::Concrete(concrete) => concrete,
            Value::Reference(reference) => reference.into_concrete(),
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Concrete(ConcreteValue::Boolean(boolean)) => Some(*boolean),
            Value::Reference(ReferenceValue::Concrete(ConcreteValue::Boolean(boolean))) => {
                Some(*boolean)
            }
            _ => None,
        }
    }
    pub fn as_function(&self) -> Option<FunctionBorrowed> {
        match self {
            Value::Concrete(ConcreteValue::Function(function)) => Some(function.as_borrowed()),
            Value::Reference(ReferenceValue::Concrete(ConcreteValue::Function(function))) => {
                Some(function.as_borrowed())
            }
            _ => None,
        }
    }

    pub fn add(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => left.add(right),
            (Value::Reference(ReferenceValue::Concrete(left)), Value::Concrete(right)) => {
                left.add(right)
            }
            (Value::Concrete(left), Value::Reference(ReferenceValue::Concrete(right))) => {
                left.add(right)
            }
            (
                Value::Reference(ReferenceValue::Concrete(left)),
                Value::Reference(ReferenceValue::Concrete(right)),
            ) => left.add(right),
            _ => Err(ValueError::CannotAdd(
                self.clone().into_concrete(),
                other.clone().into_concrete(),
            )),
        }
    }

    pub fn subtract(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => (left, right),
            (Value::Reference(left), Value::Reference(right)) => (*left, *right),
            _ => {
                return Err(ValueError::CannotSubtract(
                    self.clone().into_concrete(),
                    other.clone().into_concrete(),
                ));
            }
        };

        left.subtract(right)
    }

    pub fn multiply(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => (left, right),
            (Value::Reference(left), Value::Reference(right)) => (*left, *right),
            _ => {
                return Err(ValueError::CannotMultiply(
                    self.clone().into_concrete(),
                    other.clone().into_concrete(),
                ));
            }
        };

        left.multiply(right)
    }

    pub fn divide(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => (left, right),
            (Value::Reference(left), Value::Reference(right)) => (*left, *right),
            _ => {
                return Err(ValueError::CannotDivide(
                    self.clone().into_concrete(),
                    other.clone().into_concrete(),
                ));
            }
        };

        left.divide(right)
    }

    pub fn modulo(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => (left, right),
            (Value::Reference(left), Value::Reference(right)) => (*left, *right),
            _ => {
                return Err(ValueError::CannotModulo(
                    self.clone().into_concrete(),
                    other.clone().into_concrete(),
                ));
            }
        };

        left.modulo(right)
    }

    pub fn negate(&self) -> Result<ConcreteValue, ValueError> {
        match self {
            Value::Concrete(concrete) => concrete.negate(),
            Value::Reference(reference) => reference.negate(),
            _ => Err(ValueError::CannotNegate(self.clone().into_concrete())),
        }
    }

    pub fn not(&self) -> Result<ConcreteValue, ValueError> {
        match self {
            Value::Concrete(concrete) => concrete.not(),
            Value::Reference(reference) => reference.not(),
            _ => Err(ValueError::CannotNot(self.clone().into_concrete())),
        }
    }

    pub fn equal(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => (left, right),
            (Value::Reference(left), Value::Reference(right)) => (*left, *right),
            _ => {
                return Err(ValueError::CannotCompare(
                    self.clone().into_concrete(),
                    other.clone().into_concrete(),
                ));
            }
        };

        left.equal(right)
    }

    pub fn less_than(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => (left, right),
            (Value::Reference(left), Value::Reference(right)) => (*left, *right),
            _ => {
                return Err(ValueError::CannotCompare(
                    self.clone().into_concrete(),
                    other.clone().into_concrete(),
                ));
            }
        };

        left.less_than(right)
    }

    pub fn less_than_or_equal(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => (left, right),
            (Value::Reference(left), Value::Reference(right)) => (*left, *right),
            _ => {
                return Err(ValueError::CannotCompare(
                    self.clone().into_concrete(),
                    other.clone().into_concrete(),
                ));
            }
        };

        left.less_than_or_equal(right)
    }
}

impl<'a> Display for Value<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Concrete(concrete) => write!(f, "{}", concrete),
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
            Value::Reference(reference) => write!(f, "{}", reference),
            Value::FunctionBorrowed(function_reference) => {
                write!(f, "{}", function_reference.chunk().r#type())
            }
        }
    }
}

impl<'a> Eq for Value<'a> {}

impl<'a> PartialOrd for Value<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for Value<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => left.cmp(right),
            (Value::Concrete(_), _) => Ordering::Greater,
            (Value::List(left), Value::List(right)) => left.cmp(right),
            (Value::List(_), _) => Ordering::Greater,
            (Value::Reference(left), Value::Reference(right)) => left.cmp(right),
            (Value::Reference(_), _) => Ordering::Greater,
            (Value::FunctionBorrowed(left), Value::FunctionBorrowed(right)) => left.cmp(right),
            (Value::FunctionBorrowed(_), _) => Ordering::Greater,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ConcreteValue {
    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Function(Function),
    Integer(i64),
    List(Vec<ConcreteValue>),
    Range(RangeValue),
    String(String),
}

impl ConcreteValue {
    pub fn boolean(value: bool) -> Self {
        ConcreteValue::Boolean(value)
    }

    pub fn byte(value: u8) -> Self {
        ConcreteValue::Byte(value)
    }

    pub fn character(value: char) -> Self {
        ConcreteValue::Character(value)
    }

    pub fn float(value: f64) -> Self {
        ConcreteValue::Float(value)
    }

    pub fn function(chunk: Chunk) -> Self {
        ConcreteValue::Function(Function::new(chunk))
    }

    pub fn integer<T: Into<i64>>(into_i64: T) -> Self {
        ConcreteValue::Integer(into_i64.into())
    }

    pub fn list<T: Into<Vec<ConcreteValue>>>(into_list: T) -> Self {
        ConcreteValue::List(into_list.into())
    }

    pub fn range(range: RangeValue) -> Self {
        ConcreteValue::Range(range)
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

    pub fn to_owned_value<'a>(self) -> Value<'a> {
        Value::Concrete(self)
    }

    pub fn as_reference_value(&self) -> Value {
        Value::Reference(self)
    }

    pub fn r#type(&self) -> Type {
        match self {
            ConcreteValue::Boolean(_) => Type::Boolean,
            ConcreteValue::Byte(_) => Type::Byte,
            ConcreteValue::Character(_) => Type::Character,
            ConcreteValue::Float(_) => Type::Float,
            ConcreteValue::Function(function) => function.r#type(),
            ConcreteValue::Integer(_) => Type::Integer,
            ConcreteValue::List(list) => {
                let item_type = list.first().map_or(Type::Any, |item| item.r#type());

                Type::List {
                    item_type: Box::new(item_type),
                    length: list.len(),
                }
            }
            ConcreteValue::Range(range) => range.r#type(),
            ConcreteValue::String(string) => Type::String {
                length: Some(string.len()),
            },
        }
    }

    pub fn add(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let sum = match (self, other) {
            (Byte(left), Byte(right)) => ConcreteValue::byte(left.saturating_add(*right)),
            (Float(left), Float(right)) => ConcreteValue::float(*left + *right),
            (Integer(left), Integer(right)) => ConcreteValue::integer(left.saturating_add(*right)),
            (String(left), String(right)) => ConcreteValue::string(format!("{}{}", left, right)),
            _ => return Err(ValueError::CannotAdd(self.clone(), other.clone())),
        };

        Ok(sum)
    }

    pub fn subtract(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let difference = match (self, other) {
            (Byte(left), Byte(right)) => ConcreteValue::byte(left.saturating_sub(*right)),
            (Float(left), Float(right)) => ConcreteValue::float(left - right),
            (Integer(left), Integer(right)) => ConcreteValue::integer(left.saturating_sub(*right)),
            _ => return Err(ValueError::CannotSubtract(self.clone(), other.clone())),
        };

        Ok(difference)
    }

    pub fn multiply(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let product = match (self, other) {
            (Byte(left), Byte(right)) => ConcreteValue::byte(left.saturating_mul(*right)),
            (Float(left), Float(right)) => ConcreteValue::float(left * right),
            (Integer(left), Integer(right)) => ConcreteValue::integer(left.saturating_mul(*right)),
            _ => return Err(ValueError::CannotMultiply(self.clone(), other.clone())),
        };

        Ok(product)
    }

    pub fn divide(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let quotient = match (self, other) {
            (Byte(left), Byte(right)) => ConcreteValue::byte(left.saturating_div(*right)),
            (Float(left), Float(right)) => ConcreteValue::float(left / right),
            (Integer(left), Integer(right)) => ConcreteValue::integer(left.saturating_div(*right)),
            _ => return Err(ValueError::CannotMultiply(self.clone(), other.clone())),
        };

        Ok(quotient)
    }

    pub fn modulo(&self, other: &Self) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let product = match (self, other) {
            (Byte(left), Byte(right)) => ConcreteValue::byte(left.wrapping_rem(*right)),
            (Float(left), Float(right)) => ConcreteValue::float(left % right),
            (Integer(left), Integer(right)) => {
                ConcreteValue::integer(left.wrapping_rem_euclid(*right))
            }
            _ => return Err(ValueError::CannotMultiply(self.clone(), other.clone())),
        };

        Ok(product)
    }

    pub fn negate(&self) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let negated = match self {
            Boolean(value) => ConcreteValue::boolean(!value),
            Byte(value) => ConcreteValue::byte(value.wrapping_neg()),
            Float(value) => ConcreteValue::float(-value),
            Integer(value) => ConcreteValue::integer(value.wrapping_neg()),
            _ => return Err(ValueError::CannotNegate(self.clone())),
        };

        Ok(negated)
    }

    pub fn not(&self) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let not = match self {
            Boolean(value) => ConcreteValue::boolean(!value),
            _ => return Err(ValueError::CannotNot(self.clone())),
        };

        Ok(not)
    }

    pub fn equal(&self, other: &ConcreteValue) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let equal = match (self, other) {
            (Boolean(left), Boolean(right)) => ConcreteValue::boolean(left == right),
            (Byte(left), Byte(right)) => ConcreteValue::boolean(left == right),
            (Character(left), Character(right)) => ConcreteValue::boolean(left == right),
            (Float(left), Float(right)) => ConcreteValue::boolean(left == right),
            (Function(left), Function(right)) => ConcreteValue::boolean(left == right),
            (Integer(left), Integer(right)) => ConcreteValue::boolean(left == right),
            (List(left), List(right)) => ConcreteValue::boolean(left == right),
            (Range(left), Range(right)) => ConcreteValue::boolean(left == right),
            (String(left), String(right)) => ConcreteValue::boolean(left == right),
            _ => return Err(ValueError::CannotCompare(self.clone(), other.clone())),
        };

        Ok(equal)
    }

    pub fn less_than(&self, other: &ConcreteValue) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let less_than = match (self, other) {
            (Boolean(left), Boolean(right)) => ConcreteValue::boolean(left < right),
            (Byte(left), Byte(right)) => ConcreteValue::boolean(left < right),
            (Character(left), Character(right)) => ConcreteValue::boolean(left < right),
            (Float(left), Float(right)) => ConcreteValue::boolean(left < right),
            (Function(left), Function(right)) => ConcreteValue::boolean(left < right),
            (Integer(left), Integer(right)) => ConcreteValue::boolean(left < right),
            (List(left), List(right)) => ConcreteValue::boolean(left < right),
            (Range(left), Range(right)) => ConcreteValue::boolean(left < right),
            (String(left), String(right)) => ConcreteValue::boolean(left < right),
            _ => return Err(ValueError::CannotCompare(self.clone(), other.clone())),
        };

        Ok(less_than)
    }

    pub fn less_than_or_equal(&self, other: &ConcreteValue) -> Result<ConcreteValue, ValueError> {
        use ConcreteValue::*;

        let less_than_or_equal = match (self, other) {
            (Boolean(left), Boolean(right)) => ConcreteValue::boolean(left <= right),
            (Byte(left), Byte(right)) => ConcreteValue::boolean(left <= right),
            (Character(left), Character(right)) => ConcreteValue::boolean(left <= right),
            (Float(left), Float(right)) => ConcreteValue::boolean(left <= right),
            (Function(left), Function(right)) => ConcreteValue::boolean(left <= right),
            (Integer(left), Integer(right)) => ConcreteValue::boolean(left <= right),
            (List(left), List(right)) => ConcreteValue::boolean(left <= right),
            (Range(left), Range(right)) => ConcreteValue::boolean(left <= right),
            (String(left), String(right)) => ConcreteValue::boolean(left <= right),
            _ => return Err(ValueError::CannotCompare(self.clone(), other.clone())),
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
            ConcreteValue::Range(range) => ConcreteValue::Range(range.clone()),
            ConcreteValue::String(string) => ConcreteValue::String(string.clone()),
        }
    }
}

impl Eq for ConcreteValue {}

impl PartialOrd for ConcreteValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ConcreteValue {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (ConcreteValue::Boolean(left), ConcreteValue::Boolean(right)) => left.cmp(right),
            (ConcreteValue::Boolean(_), _) => Ordering::Greater,
            (ConcreteValue::Byte(left), ConcreteValue::Byte(right)) => left.cmp(right),
            (ConcreteValue::Byte(_), _) => Ordering::Greater,
            (ConcreteValue::Character(left), ConcreteValue::Character(right)) => left.cmp(right),
            (ConcreteValue::Character(_), _) => Ordering::Greater,
            (ConcreteValue::Float(left), ConcreteValue::Float(right)) => {
                left.to_bits().cmp(&right.to_bits())
            }
            (ConcreteValue::Float(_), _) => Ordering::Greater,
            (ConcreteValue::Function(left), ConcreteValue::Function(right)) => left.cmp(right),
            (ConcreteValue::Function(_), _) => Ordering::Greater,
            (ConcreteValue::Integer(left), ConcreteValue::Integer(right)) => left.cmp(right),
            (ConcreteValue::Integer(_), _) => Ordering::Greater,
            (ConcreteValue::List(left), ConcreteValue::List(right)) => left.cmp(right),
            (ConcreteValue::List(_), _) => Ordering::Greater,
            (ConcreteValue::Range(left), ConcreteValue::Range(right)) => left.cmp(right),
            (ConcreteValue::Range(_), _) => Ordering::Greater,
            (ConcreteValue::String(left), ConcreteValue::String(right)) => left.cmp(right),
            (ConcreteValue::String(_), _) => Ordering::Greater,
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
            ConcreteValue::Function(function) => write!(f, "{function}"),
            ConcreteValue::Integer(integer) => write!(f, "{integer}"),
            ConcreteValue::List(list) => {
                write!(f, "[")?;

                for (index, element) in list.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{element}")?;
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

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RangeValue {
    ByteRange { start: u8, end: u8 },
    ByteRangeInclusive { start: u8, end: u8 },
    CharacterRange { start: char, end: char },
    CharacterRangeInclusive { start: char, end: char },
    FloatRange { start: f64, end: f64 },
    FloatRangeInclusive { start: f64, end: f64 },
    IntegerRange { start: i64, end: i64 },
    IntegerRangeInclusive { start: i64, end: i64 },
}

impl RangeValue {
    pub fn r#type(&self) -> Type {
        let inner_type = match self {
            RangeValue::ByteRange { .. } | RangeValue::ByteRangeInclusive { .. } => Type::Byte,
            RangeValue::CharacterRange { .. } | RangeValue::CharacterRangeInclusive { .. } => {
                Type::Character
            }
            RangeValue::FloatRange { .. } | RangeValue::FloatRangeInclusive { .. } => Type::Float,
            RangeValue::IntegerRange { .. } | RangeValue::IntegerRangeInclusive { .. } => {
                Type::Integer
            }
        };

        Type::Range {
            r#type: Box::new(inner_type),
        }
    }
}

impl From<Range<u8>> for RangeValue {
    fn from(range: Range<u8>) -> Self {
        RangeValue::ByteRange {
            start: range.start,
            end: range.end,
        }
    }
}

impl From<RangeInclusive<u8>> for RangeValue {
    fn from(range: RangeInclusive<u8>) -> Self {
        RangeValue::ByteRangeInclusive {
            start: *range.start(),
            end: *range.end(),
        }
    }
}

impl From<Range<char>> for RangeValue {
    fn from(range: Range<char>) -> Self {
        RangeValue::CharacterRange {
            start: range.start,
            end: range.end,
        }
    }
}

impl From<RangeInclusive<char>> for RangeValue {
    fn from(range: RangeInclusive<char>) -> Self {
        RangeValue::CharacterRangeInclusive {
            start: *range.start(),
            end: *range.end(),
        }
    }
}

impl From<Range<f64>> for RangeValue {
    fn from(range: Range<f64>) -> Self {
        RangeValue::FloatRange {
            start: range.start,
            end: range.end,
        }
    }
}

impl From<RangeInclusive<f64>> for RangeValue {
    fn from(range: RangeInclusive<f64>) -> Self {
        RangeValue::FloatRangeInclusive {
            start: *range.start(),
            end: *range.end(),
        }
    }
}

impl From<Range<i32>> for RangeValue {
    fn from(range: Range<i32>) -> Self {
        RangeValue::IntegerRange {
            start: range.start as i64,
            end: range.end as i64,
        }
    }
}

impl From<RangeInclusive<i32>> for RangeValue {
    fn from(range: RangeInclusive<i32>) -> Self {
        RangeValue::IntegerRangeInclusive {
            start: *range.start() as i64,
            end: *range.end() as i64,
        }
    }
}

impl From<Range<i64>> for RangeValue {
    fn from(range: Range<i64>) -> Self {
        RangeValue::IntegerRange {
            start: range.start,
            end: range.end,
        }
    }
}

impl From<RangeInclusive<i64>> for RangeValue {
    fn from(range: RangeInclusive<i64>) -> Self {
        RangeValue::IntegerRangeInclusive {
            start: *range.start(),
            end: *range.end(),
        }
    }
}

impl Display for RangeValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            RangeValue::ByteRange { start, end } => write!(f, "{}..{}", start, end),
            RangeValue::ByteRangeInclusive { start, end } => {
                write!(f, "{}..={}", start, end)
            }
            RangeValue::CharacterRange { start, end } => {
                write!(f, "{}..{}", start, end)
            }
            RangeValue::CharacterRangeInclusive { start, end } => {
                write!(f, "{}..={}", start, end)
            }
            RangeValue::FloatRange { start, end } => write!(f, "{}..{}", start, end),
            RangeValue::FloatRangeInclusive { start, end } => {
                write!(f, "{}..={}", start, end)
            }
            RangeValue::IntegerRange { start, end } => write!(f, "{}..{}", start, end),
            RangeValue::IntegerRangeInclusive { start, end } => {
                write!(f, "{}..={}", start, end)
            }
        }
    }
}

impl Eq for RangeValue {}

impl PartialOrd for RangeValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RangeValue {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                RangeValue::ByteRange {
                    start: left_start,
                    end: left_end,
                },
                RangeValue::ByteRange {
                    start: right_start,
                    end: right_end,
                },
            ) => {
                let start_cmp = left_start.cmp(right_start);

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left_end.cmp(right_end)
                }
            }
            (RangeValue::ByteRange { .. }, _) => Ordering::Greater,
            (
                RangeValue::ByteRangeInclusive {
                    start: left_start,
                    end: left_end,
                },
                RangeValue::ByteRangeInclusive {
                    start: right_start,
                    end: right_end,
                },
            ) => {
                let start_cmp = left_start.cmp(&right_start);

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left_end.cmp(&right_end)
                }
            }
            (RangeValue::ByteRangeInclusive { .. }, _) => Ordering::Greater,
            (
                RangeValue::CharacterRange {
                    start: left_start,
                    end: left_end,
                },
                RangeValue::CharacterRange {
                    start: right_start,
                    end: right_end,
                },
            ) => {
                let start_cmp = left_start.cmp(right_start);

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left_end.cmp(right_end)
                }
            }
            (RangeValue::CharacterRange { .. }, _) => Ordering::Greater,
            (
                RangeValue::CharacterRangeInclusive {
                    start: left_start,
                    end: left_end,
                },
                RangeValue::CharacterRangeInclusive {
                    start: right_start,
                    end: right_end,
                },
            ) => {
                let start_cmp = left_start.cmp(right_start);

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left_end.cmp(right_end)
                }
            }
            (RangeValue::CharacterRangeInclusive { .. }, _) => Ordering::Greater,
            (
                RangeValue::FloatRange {
                    start: left_start,
                    end: left_end,
                },
                RangeValue::FloatRange {
                    start: right_start,
                    end: right_end,
                },
            ) => {
                let start_cmp = left_start.partial_cmp(right_start).unwrap();

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left_end.partial_cmp(right_end).unwrap()
                }
            }
            (RangeValue::FloatRange { .. }, _) => Ordering::Greater,
            (
                RangeValue::FloatRangeInclusive {
                    start: left_start,
                    end: left_end,
                },
                RangeValue::FloatRangeInclusive {
                    start: right_start,
                    end: right_end,
                },
            ) => {
                let start_cmp = left_start.partial_cmp(right_start).unwrap();

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left_end.partial_cmp(right_end).unwrap()
                }
            }
            (RangeValue::FloatRangeInclusive { .. }, _) => Ordering::Greater,
            (
                RangeValue::IntegerRange {
                    start: left_start,
                    end: left_end,
                },
                RangeValue::IntegerRange {
                    start: right_start,
                    end: right_end,
                },
            ) => {
                let start_cmp = left_start.cmp(right_start);

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left_end.cmp(right_end)
                }
            }
            (RangeValue::IntegerRange { .. }, _) => Ordering::Greater,
            (
                RangeValue::IntegerRangeInclusive {
                    start: left_start,
                    end: left_end,
                },
                RangeValue::IntegerRangeInclusive {
                    start: right_start,
                    end: right_end,
                },
            ) => {
                let start_cmp = left_start.cmp(right_start);

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left_end.cmp(right_end)
                }
            }
            (RangeValue::IntegerRangeInclusive { .. }, _) => Ordering::Greater,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueError {
    CannotAdd(ConcreteValue, ConcreteValue),
    CannotAnd(ConcreteValue, ConcreteValue),
    CannotCompare(ConcreteValue, ConcreteValue),
    CannotDivide(ConcreteValue, ConcreteValue),
    CannotModulo(ConcreteValue, ConcreteValue),
    CannotMultiply(ConcreteValue, ConcreteValue),
    CannotNegate(ConcreteValue),
    CannotNot(ConcreteValue),
    CannotSubtract(ConcreteValue, ConcreteValue),
    CannotOr(ConcreteValue, ConcreteValue),
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
