use std::array;

use crate::{
    AbstractList, Address, Chunk, DustString, instruction::AddressKind, value::AbstractFunction,
};

#[derive(Debug)]
pub struct Memory {
    pub booleans: Vec<Slot<bool>>,
    pub bytes: Vec<Slot<u8>>,
    pub characters: Vec<Slot<char>>,
    pub floats: Vec<Slot<f64>>,
    pub integers: Vec<Slot<i64>>,
    pub strings: Vec<Slot<DustString>>,
    pub lists: Vec<Slot<AbstractList>>,
    pub functions: Vec<Slot<AbstractFunction>>,

    pub registers: RegisterTable,
}

impl Memory {
    pub fn new(chunk: &Chunk) -> Self {
        Memory {
            booleans: vec![Slot::new(false); chunk.boolean_memory_length as usize],
            bytes: vec![Slot::new(0); chunk.byte_memory_length as usize],
            characters: vec![Slot::new(char::default()); chunk.character_memory_length as usize],
            floats: vec![Slot::new(0.0); chunk.float_memory_length as usize],
            integers: vec![Slot::new(0); chunk.integer_memory_length as usize],
            strings: vec![Slot::new(DustString::new()); chunk.string_memory_length as usize],
            lists: vec![Slot::new(AbstractList::default()); chunk.list_memory_length as usize],
            functions: vec![
                Slot::new(AbstractFunction::default());
                chunk.function_memory_length as usize
            ],
            registers: RegisterTable::new(),
        }
    }
}

#[derive(Debug)]
pub struct RegisterTable<const LENGTH: usize = 3> {
    pub booleans: [bool; LENGTH],
    pub bytes: [u8; LENGTH],
    pub characters: [char; LENGTH],
    pub floats: [f64; LENGTH],
    pub integers: [i64; LENGTH],
    pub strings: [DustString; LENGTH],
    pub lists: [AbstractList; LENGTH],
    pub functions: [AbstractFunction; LENGTH],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_table_is_well_aligned() {
        use std::mem::align_of;
        use std::mem::size_of;

        assert_eq!(align_of::<RegisterTable>(), 8);
        assert_eq!(size_of::<RegisterTable>(), 8 * 8);
    }
}

impl<const LENGTH: usize> RegisterTable<LENGTH> {
    pub fn new() -> Self {
        RegisterTable {
            booleans: [false; LENGTH],
            bytes: [0; LENGTH],
            characters: [char::default(); LENGTH],
            floats: [0.0; LENGTH],
            integers: [0; LENGTH],
            strings: array::from_fn(|_| DustString::new()),
            lists: array::from_fn(|_| AbstractList {
                item_pointers: Vec::with_capacity(0),
            }),
            functions: [AbstractFunction {
                prototype_address: Address::new(0, AddressKind(0)),
            }; LENGTH],
        }
    }
}

impl<const LENGTH: usize> Default for RegisterTable<LENGTH> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Slot<T> {
    value: T,
    is_closed: bool,
}

impl<T> Slot<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            is_closed: false,
        }
    }

    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    pub fn close(&mut self) {
        self.is_closed = true;
    }

    pub fn set(&mut self, new_value: T) {
        self.value = new_value;
    }

    pub fn as_value(&self) -> &T {
        &self.value
    }

    pub fn as_value_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: Copy> Slot<T> {
    pub fn copy_value(&self) -> T {
        self.value
    }
}

impl<T: Clone> Slot<T> {
    pub fn clone_value(&self) -> T {
        self.value.clone()
    }
}

impl<T: Default> Default for Slot<T> {
    fn default() -> Self {
        Self {
            value: Default::default(),
            is_closed: false,
        }
    }
}
