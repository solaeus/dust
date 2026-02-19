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

    pub fn encoded_boolean(boolean: bool) -> Self {
        Address {
            index: boolean as u16,
            memory: MemoryKind::ENCODED,
        }
    }

    pub fn decode_boolean(&self) -> bool {
        self.index != 0
    }

    pub fn encoded_byte(byte: u8) -> Self {
        Address {
            index: byte as u16,
            memory: MemoryKind::ENCODED,
        }
    }

    pub fn decode_byte(&self) -> u8 {
        self.index as u8
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoded_booleans() {
        let true_address = Address::encoded_boolean(true);
        let false_address = Address::encoded_boolean(false);

        assert!(true_address.decode_boolean());
        assert!(!false_address.decode_boolean());
    }

    #[test]
    fn encoded_bytes() {
        let byte: u8 = 123;
        let address = Address::encoded_byte(byte);

        assert_eq!(address.decode_byte(), byte);
    }
}
