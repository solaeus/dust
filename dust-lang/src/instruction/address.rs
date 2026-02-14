use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use super::MemoryKind;

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Address {
    pub index: u16,
    pub memory: MemoryKind,
}

impl Address {
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

    pub fn encoded(index: u16) -> Self {
        Address {
            index,
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
