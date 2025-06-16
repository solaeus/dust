use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::Type;

/// An ordered sequence of values.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum DustRange {
    Byte(DustRangeInner<u8>),
    Character(DustRangeInner<char>),
    Float(DustRangeInner<f64>),
    Integer(DustRangeInner<i64>),
}

impl DustRange {
    pub fn r#type(&self) -> Type {
        match self {
            DustRange::Byte(_) => Type::Range(Box::new(Type::Byte)),
            DustRange::Character(_) => Type::Range(Box::new(Type::Character)),
            DustRange::Float(_) => Type::Range(Box::new(Type::Float)),
            DustRange::Integer(_) => Type::Range(Box::new(Type::Integer)),
        }
    }
}

impl Display for DustRange {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DustRange::Byte(byte_range) => write!(f, "{byte_range}"),
            DustRange::Character(character_range) => write!(f, "{character_range}"),
            DustRange::Float(float_range) => write!(f, "{float_range}"),
            DustRange::Integer(integer_range) => write!(f, "{integer_range}"),
        }
    }
}

/// An ordered sequence of values. These variants mirror the range types in `std::range`. This type
/// is not used on its own but forms the basis for Dust's [`ConcreteRange`] values.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum DustRangeInner<T> {
    FromStart { start: T },
    Full,
    Inclusive { start: T, end: T },
    SemiInclusive { start: T, end: T },
    ToEnd { end: T },
    ToEndInclusive { end: T },
}

impl<T: Display> Display for DustRangeInner<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DustRangeInner::FromStart { start } => write!(f, "{start}.."),
            DustRangeInner::Full => write!(f, ".."),
            DustRangeInner::Inclusive { start, end } => write!(f, "{start}..={end}"),
            DustRangeInner::SemiInclusive { start, end } => write!(f, "{start}..{end}"),
            DustRangeInner::ToEnd { end } => write!(f, "..{end}"),
            DustRangeInner::ToEndInclusive { end } => write!(f, "..={end}"),
        }
    }
}
