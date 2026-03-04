use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use super::MemoryKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Address {
    pub index: u16,
    pub memory: MemoryKind,
}

impl Address {
    pub fn none() -> Self {
        Address {
            index: u16::MAX,
            memory: MemoryKind::ENCODED,
        }
    }

    pub fn constant(index: u16) -> Self {
        Address {
            index,
            memory: MemoryKind::CONSTANT,
        }
    }

    pub fn register(index: u16) -> Self {
        Address {
            index,
            memory: MemoryKind::REGISTER,
        }
    }

    pub fn encoded_boolean(boolean: bool) -> Self {
        Address {
            index: boolean as u16,
            memory: MemoryKind::ENCODED,
        }
    }

    pub fn encoded_u8(u8: u8) -> Self {
        Address {
            index: u8 as u16,
            memory: MemoryKind::ENCODED,
        }
    }

    pub fn encoded_i8(i8: i8) -> Self {
        Address {
            index: i8 as u16,
            memory: MemoryKind::ENCODED,
        }
    }

    pub fn encoded_u16(u16: u16) -> Self {
        Address {
            index: u16,
            memory: MemoryKind::ENCODED,
        }
    }

    pub fn encoded_i16(i16: i16) -> Self {
        Address {
            index: i16 as u16,
            memory: MemoryKind::ENCODED,
        }
    }

    pub fn prototype(index: u16) -> Self {
        Address {
            index,
            memory: MemoryKind::PROTOTYPE,
        }
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Address { index, memory } = self;

        write!(f, "{memory}_{index}")
    }
}
