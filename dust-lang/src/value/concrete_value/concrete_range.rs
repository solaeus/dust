use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Type, value::DustRange};

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
            ConcreteRange::Byte(dust_range) => Type::Range(Box::new(Type::Byte)),
            ConcreteRange::Character(dust_range) => todo!(),
            ConcreteRange::Float(dust_range) => todo!(),
            ConcreteRange::Integer(dust_range) => todo!(),
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
