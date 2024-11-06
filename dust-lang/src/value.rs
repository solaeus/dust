//! Dust value representation
//!
//! # Examples
//!
//! Each type of value has a corresponding method for instantiation:
//!
//! ```
//! # use dust_lang::Value;
//! let boolean = Value::boolean(true);
//! let float = Value::float(3.14);
//! let integer = Value::integer(42);
//! let string = Value::string("Hello, world!");
//! ```
//!
//! Values have a type, which can be retrieved using the `r#type` method:
//!
//! ```
//! # use dust_lang::*;
//! let value = Value::integer(42);
//!
//! assert_eq!(value.r#type(), Type::Integer);
//! ```
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter},
    ops::{Range, RangeInclusive},
};

use serde::{Deserialize, Serialize};

use crate::{Chunk, FunctionType, RangeableType, Span, Type, Vm, VmError};

/// Dust value representation
///
/// See the [module-level documentation][self] for more.
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Value {
    Concrete(ConcreteValue),
    Abstract(AbstractValue),
}

impl Value {
    pub fn boolean(value: bool) -> Self {
        Value::Concrete(ConcreteValue::Boolean(value))
    }

    pub fn byte(value: u8) -> Self {
        Value::Concrete(ConcreteValue::Byte(value))
    }

    pub fn character(value: char) -> Self {
        Value::Concrete(ConcreteValue::Character(value))
    }

    pub fn float(value: f64) -> Self {
        Value::Concrete(ConcreteValue::Float(value))
    }

    pub fn function(body: Chunk, r#type: FunctionType) -> Self {
        Value::Concrete(ConcreteValue::Function(Function {
            chunk: body,
            r#type: Type::Function(r#type),
        }))
    }

    pub fn integer<T: Into<i64>>(into_i64: T) -> Self {
        Value::Concrete(ConcreteValue::Integer(into_i64.into()))
    }

    pub fn list<T: Into<Vec<Value>>>(items: T) -> Self {
        Value::Concrete(ConcreteValue::List(items.into()))
    }

    pub fn abstract_list(start: u8, end: u8, item_type: Type) -> Self {
        Value::Abstract(AbstractValue::List {
            start,
            end,
            item_type,
        })
    }

    pub fn string<T: ToString>(to_string: T) -> Self {
        Value::Concrete(ConcreteValue::String(to_string.to_string()))
    }

    pub fn as_string(&self) -> Option<&String> {
        if let Value::Concrete(ConcreteValue::String(string)) = self {
            Some(string)
        } else {
            None
        }
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Value::Concrete(ConcreteValue::Function(_)))
    }

    pub fn r#type(&self) -> Type {
        match self {
            Value::Concrete(data) => data.r#type(),
            Value::Abstract(AbstractValue::List {
                start,
                end,
                item_type,
            }) => {
                let length = (end - start + 1) as usize;

                Type::List {
                    length,
                    item_type: Box::new(item_type.clone()),
                }
            }
        }
    }

    pub fn add(&self, other: &Value) -> Result<Value, ValueError> {
        use ConcreteValue::*;
        use Value::*;

        let sum = match (self, other) {
            (Concrete(Byte(left)), Concrete(Byte(right))) => {
                Value::byte(left.saturating_add(*right))
            }
            (Concrete(Float(left)), Concrete(Float(right))) => Value::float(left + right),
            (Concrete(Integer(left)), Concrete(Integer(right))) => {
                Value::integer(left.saturating_add(*right))
            }
            (Concrete(String(left)), Concrete(String(right))) => {
                Value::string(format!("{}{}", left, right))
            }
            _ => return Err(ValueError::CannotAdd(self.clone(), other.clone())),
        };

        Ok(sum)
    }

    pub fn subtract(&self, other: &Value) -> Result<Value, ValueError> {
        use ConcreteValue::*;
        use Value::*;

        let different = match (self, other) {
            (Concrete(Byte(left)), Concrete(Byte(right))) => {
                Value::byte(left.saturating_sub(*right))
            }
            (Concrete(Float(left)), Concrete(Float(right))) => Value::float(left - right),
            (Concrete(Integer(left)), Concrete(Integer(right))) => {
                Value::integer(left.saturating_sub(*right))
            }
            _ => return Err(ValueError::CannotSubtract(self.clone(), other.clone())),
        };

        Ok(different)
    }

    pub fn multiply(&self, other: &Value) -> Result<Value, ValueError> {
        use ConcreteValue::*;
        use Value::*;

        let product = match (self, other) {
            (Concrete(Byte(left)), Concrete(Byte(right))) => {
                Value::byte(left.saturating_mul(*right))
            }
            (Concrete(Float(left)), Concrete(Float(right))) => Value::float(left * right),
            (Concrete(Integer(left)), Concrete(Integer(right))) => {
                Value::integer(left.saturating_mul(*right))
            }
            _ => return Err(ValueError::CannotAdd(self.clone(), other.clone())),
        };

        Ok(product)
    }

    pub fn divide(&self, other: &Value) -> Result<Value, ValueError> {
        use ConcreteValue::*;
        use Value::*;

        let product = match (self, other) {
            (Concrete(Byte(left)), Concrete(Byte(right))) => {
                Value::byte(left.saturating_div(*right))
            }
            (Concrete(Float(left)), Concrete(Float(right))) => Value::float(left / right),
            (Concrete(Integer(left)), Concrete(Integer(right))) => {
                Value::integer(left.saturating_div(*right))
            }
            _ => return Err(ValueError::CannotDivide(self.clone(), other.clone())),
        };

        Ok(product)
    }

    pub fn modulo(&self, other: &Value) -> Result<Value, ValueError> {
        use ConcreteValue::*;
        use Value::*;

        let product = match (self, other) {
            (Concrete(Byte(left)), Concrete(Byte(right))) => Value::byte(left % right),
            (Concrete(Float(left)), Concrete(Float(right))) => Value::float(left % right),
            (Concrete(Integer(left)), Concrete(Integer(right))) => Value::integer(left % right),
            _ => return Err(ValueError::CannotModulo(self.clone(), other.clone())),
        };

        Ok(product)
    }

    pub fn less_than(&self, other: &Value) -> Result<Value, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => (left, right),
            _ => return Err(ValueError::CannotCompare(self.clone(), other.clone())),
        };

        Ok(Value::boolean(left < right))
    }

    pub fn less_than_or_equal(&self, other: &Value) -> Result<Value, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => (left, right),
            _ => return Err(ValueError::CannotCompare(self.clone(), other.clone())),
        };

        Ok(Value::boolean(left <= right))
    }

    pub fn equal(&self, other: &Value) -> Result<Value, ValueError> {
        let (left, right) = match (self, other) {
            (Value::Concrete(left), Value::Concrete(right)) => (left, right),
            _ => return Err(ValueError::CannotCompare(self.clone(), other.clone())),
        };

        Ok(Value::boolean(left == right))
    }

    pub fn negate(&self) -> Result<Value, ValueError> {
        use ConcreteValue::*;
        use Value::*;

        let negated = match self {
            Concrete(Integer(integer)) => Value::integer(-integer),
            Concrete(Float(float)) => Value::float(-float),
            _ => return Err(ValueError::CannotNot(self.clone())),
        };

        Ok(negated)
    }

    pub fn not(&self) -> Result<Value, ValueError> {
        use ConcreteValue::*;
        use Value::*;

        let not = match self {
            Concrete(Boolean(boolean)) => Value::boolean(!boolean),
            Concrete(Byte(byte)) => Value::byte(!byte),
            Concrete(Integer(integer)) => Value::integer(!integer),
            _ => return Err(ValueError::CannotNot(self.clone())),
        };

        Ok(not)
    }

    pub fn to_concrete(self, vm: &mut Vm, position: Span) -> Result<Value, VmError> {
        match self {
            Value::Concrete(_) => Ok(self),
            Value::Abstract(AbstractValue::List { start, end, .. }) => {
                let mut items = Vec::new();

                for register_index in start..end {
                    let get_value = vm.empty_register(register_index, position);

                    if let Ok(value) = get_value {
                        items.push(value);
                    }
                }

                Ok(Value::Concrete(ConcreteValue::List(items)))
            }
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::boolean(value)
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value::byte(value)
    }
}

impl From<char> for Value {
    fn from(value: char) -> Self {
        Value::character(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::float(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::integer(value as i64)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::integer(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::string(value)
    }
}

impl From<&str> for Value {
    fn from(str: &str) -> Self {
        Value::string(str)
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        log::trace!("Cloning value {self}");

        match self {
            Value::Abstract(object) => Value::Abstract(object.clone()),
            Value::Concrete(concrete) => Value::Concrete(concrete.clone()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Abstract(object) => write!(f, "{object}"),
            Value::Concrete(concrete) => write!(f, "{concrete}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum ConcreteValue {
    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Function(Function),
    Integer(i64),
    List(Vec<Value>),
    Range(RangeValue),
    String(String),
}

impl ConcreteValue {
    pub fn r#type(&self) -> Type {
        match self {
            ConcreteValue::Boolean(_) => Type::Boolean,
            ConcreteValue::Byte(_) => Type::Byte,
            ConcreteValue::Character(_) => Type::Character,
            ConcreteValue::Float(_) => Type::Float,
            ConcreteValue::Function(Function { r#type, .. }) => r#type.clone(),
            ConcreteValue::Integer(_) => Type::Integer,
            ConcreteValue::List(list) => Type::List {
                item_type: list
                    .first()
                    .map(|value| Box::new(value.r#type()))
                    .unwrap_or_else(|| Box::new(Type::Any)),
                length: list.len(),
            },
            ConcreteValue::Range(range) => range.r#type(),
            ConcreteValue::String(string) => Type::String {
                length: Some(string.len()),
            },
        }
    }

    pub fn is_rangeable(&self) -> bool {
        matches!(
            self,
            ConcreteValue::Integer(_)
                | ConcreteValue::Float(_)
                | ConcreteValue::Character(_)
                | ConcreteValue::Byte(_)
        )
    }
}

impl Display for ConcreteValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
            ConcreteValue::Function(Function { r#type, .. }) => {
                write!(f, "{}", r#type)
            }
            ConcreteValue::Integer(integer) => write!(f, "{integer}"),
            ConcreteValue::List(items) => {
                write!(f, "[")?;

                for (index, item) in items.iter().enumerate() {
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
                if left.is_nan() && right.is_nan() {
                    Ordering::Equal
                } else if left.is_nan() {
                    Ordering::Less
                } else if right.is_nan() {
                    Ordering::Greater
                } else {
                    left.partial_cmp(right).unwrap()
                }
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

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Function {
    chunk: Chunk,
    r#type: Type,
}

impl Function {
    pub fn new(chunk: Chunk, r#type: Type) -> Self {
        Self { chunk, r#type }
    }

    pub fn chunk(&self) -> &Chunk {
        &self.chunk
    }

    pub fn take_chunk(self) -> Chunk {
        self.chunk
    }

    pub fn r#type(&self) -> &Type {
        &self.r#type
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.r#type)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RangeValue {
    ByteRange(Range<u8>),
    ByteRangeInclusive(RangeInclusive<u8>),
    CharacterRange(Range<char>),
    CharacterRangeInclusive(RangeInclusive<char>),
    FloatRange(Range<f64>),
    FloatRangeInclusive(RangeInclusive<f64>),
    IntegerRange(Range<i64>),
    IntegerRangeInclusive(RangeInclusive<i64>),
}

impl RangeValue {
    pub fn r#type(&self) -> Type {
        let inner_type = match self {
            RangeValue::ByteRange(_) => RangeableType::Byte,
            RangeValue::ByteRangeInclusive(_) => RangeableType::Byte,
            RangeValue::CharacterRange(_) => RangeableType::Character,
            RangeValue::CharacterRangeInclusive(_) => RangeableType::Character,
            RangeValue::FloatRange(_) => RangeableType::Float,
            RangeValue::FloatRangeInclusive(_) => RangeableType::Float,
            RangeValue::IntegerRange(_) => RangeableType::Integer,
            RangeValue::IntegerRangeInclusive(_) => RangeableType::Integer,
        };

        Type::Range { r#type: inner_type }
    }
}

impl From<Range<u8>> for RangeValue {
    fn from(range: Range<u8>) -> Self {
        RangeValue::ByteRange(range)
    }
}

impl From<RangeInclusive<u8>> for RangeValue {
    fn from(range: RangeInclusive<u8>) -> Self {
        RangeValue::ByteRangeInclusive(range)
    }
}

impl From<Range<char>> for RangeValue {
    fn from(range: Range<char>) -> Self {
        RangeValue::CharacterRange(range)
    }
}

impl From<RangeInclusive<char>> for RangeValue {
    fn from(range: RangeInclusive<char>) -> Self {
        RangeValue::CharacterRangeInclusive(range)
    }
}

impl From<Range<f64>> for RangeValue {
    fn from(range: Range<f64>) -> Self {
        RangeValue::FloatRange(range)
    }
}

impl From<RangeInclusive<f64>> for RangeValue {
    fn from(range: RangeInclusive<f64>) -> Self {
        RangeValue::FloatRangeInclusive(range)
    }
}

impl From<Range<i32>> for RangeValue {
    fn from(range: Range<i32>) -> Self {
        RangeValue::IntegerRange(range.start as i64..range.end as i64)
    }
}

impl From<RangeInclusive<i32>> for RangeValue {
    fn from(range: RangeInclusive<i32>) -> Self {
        RangeValue::IntegerRangeInclusive(*range.start() as i64..=*range.end() as i64)
    }
}

impl From<Range<i64>> for RangeValue {
    fn from(range: Range<i64>) -> Self {
        RangeValue::IntegerRange(range)
    }
}

impl From<RangeInclusive<i64>> for RangeValue {
    fn from(range: RangeInclusive<i64>) -> Self {
        RangeValue::IntegerRangeInclusive(range)
    }
}

impl Display for RangeValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            RangeValue::ByteRange(range) => write!(f, "{}..{}", range.start, range.end),
            RangeValue::ByteRangeInclusive(range) => {
                write!(f, "{}..={}", range.start(), range.end())
            }
            RangeValue::CharacterRange(range) => write!(f, "{}..{}", range.start, range.end),
            RangeValue::CharacterRangeInclusive(range) => {
                write!(f, "{}..={}", range.start(), range.end())
            }
            RangeValue::FloatRange(range) => write!(f, "{}..{}", range.start, range.end),
            RangeValue::FloatRangeInclusive(range) => {
                write!(f, "{}..={}", range.start(), range.end())
            }
            RangeValue::IntegerRange(range) => write!(f, "{}..{}", range.start, range.end),
            RangeValue::IntegerRangeInclusive(range) => {
                write!(f, "{}..={}", range.start(), range.end())
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
            (RangeValue::ByteRange(left), RangeValue::ByteRange(right)) => {
                let start_cmp = left.start.cmp(&right.start);

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left.end.cmp(&right.end)
                }
            }
            (RangeValue::ByteRange(_), _) => Ordering::Greater,
            (RangeValue::ByteRangeInclusive(left), RangeValue::ByteRangeInclusive(right)) => {
                let start_cmp = left.start().cmp(right.start());

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left.end().cmp(right.end())
                }
            }
            (RangeValue::ByteRangeInclusive(_), _) => Ordering::Greater,
            (RangeValue::CharacterRange(left), RangeValue::CharacterRange(right)) => {
                let start_cmp = left.start.cmp(&right.start);

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left.end.cmp(&right.end)
                }
            }
            (RangeValue::CharacterRange(_), _) => Ordering::Greater,
            (
                RangeValue::CharacterRangeInclusive(left),
                RangeValue::CharacterRangeInclusive(right),
            ) => {
                let start_cmp = left.start().cmp(right.start());

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left.end().cmp(right.end())
                }
            }
            (RangeValue::CharacterRangeInclusive(_), _) => Ordering::Greater,
            (RangeValue::FloatRange(left), RangeValue::FloatRange(right)) => {
                let start_cmp = left.start.to_bits().cmp(&right.start.to_bits());

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left.end.to_bits().cmp(&right.end.to_bits())
                }
            }
            (RangeValue::FloatRange(_), _) => Ordering::Greater,
            (RangeValue::FloatRangeInclusive(left), RangeValue::FloatRangeInclusive(right)) => {
                let start_cmp = left.start().to_bits().cmp(&right.start().to_bits());

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left.end().to_bits().cmp(&right.end().to_bits())
                }
            }
            (RangeValue::FloatRangeInclusive(_), _) => Ordering::Greater,
            (RangeValue::IntegerRange(left), RangeValue::IntegerRange(right)) => {
                let start_cmp = left.start.cmp(&right.start);

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left.end.cmp(&right.end)
                }
            }
            (RangeValue::IntegerRange(_), _) => Ordering::Greater,
            (RangeValue::IntegerRangeInclusive(left), RangeValue::IntegerRangeInclusive(right)) => {
                let start_cmp = left.start().cmp(right.start());

                if start_cmp != Ordering::Equal {
                    start_cmp
                } else {
                    left.end().cmp(right.end())
                }
            }
            (RangeValue::IntegerRangeInclusive(_), _) => Ordering::Greater,
        }
    }
}

/// Value representation that can be resolved to a concrete value by the VM.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AbstractValue {
    List { start: u8, end: u8, item_type: Type },
}

impl Display for AbstractValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AbstractValue::List { start, end, .. } => {
                write!(f, "List [R{}..=R{}]", start, end)
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
