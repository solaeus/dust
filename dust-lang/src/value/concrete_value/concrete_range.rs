use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::Type;

/// An ordered sequence of values.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ConcreteRange {
    Byte(DustRange<u8>),
    Character(DustRange<char>),
    Float(DustRange<f64>),
    Integer(DustRange<i64>),
}

impl ConcreteRange {
    pub fn r#type(&self) -> Type {
        match self {
            ConcreteRange::Byte(_) => Type::Range(Box::new(Type::Byte)),
            ConcreteRange::Character(_) => Type::Range(Box::new(Type::Character)),
            ConcreteRange::Float(_) => Type::Range(Box::new(Type::Float)),
            ConcreteRange::Integer(_) => Type::Range(Box::new(Type::Integer)),
        }
    }
}

impl Display for ConcreteRange {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ConcreteRange::Byte(byte_range) => write!(f, "{byte_range}"),
            ConcreteRange::Character(character_range) => write!(f, "{character_range}"),
            ConcreteRange::Float(float_range) => write!(f, "{float_range}"),
            ConcreteRange::Integer(integer_range) => write!(f, "{integer_range}"),
        }
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
