use std::{
    fmt::{self, Display, Formatter},
    ops::{Range, RangeInclusive},
};

use serde::{Deserialize, Serialize};

use crate::Type;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
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
