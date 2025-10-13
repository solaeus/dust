use std::{
    hash::{Hash, Hasher},
    ops::Range,
};

use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHasher};
use serde::{Deserialize, Serialize};

use crate::instruction::{Address, OperandType};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConstantTable {
    payloads: IndexMap<u64, u64, FxBuildHasher>,
    tags: Vec<OperandType>,
    string_pool: Vec<u8>,
    array_pool: Vec<Address>,
}

impl ConstantTable {
    pub fn new() -> Self {
        Self {
            payloads: IndexMap::default(),
            tags: Vec::new(),
            string_pool: Vec::new(),
            array_pool: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.payloads.len()
    }

    pub fn is_empty(&self) -> bool {
        self.payloads.is_empty()
    }

    pub fn iter<'a>(&'a self) -> ConstantTableIterator<'a> {
        ConstantTableIterator {
            table: self,
            index: 0,
        }
    }

    pub fn get_string_pool_range(&self, range: Range<usize>) -> &str {
        unsafe { str::from_utf8_unchecked(self.string_pool.get(range).unwrap_or_default()) }
    }

    pub fn finalize_string_pool(&mut self) {
        let mut new_string_pool = Vec::with_capacity(self.string_pool.len());

        for (payload, tag) in self.payloads.values_mut().zip(self.tags.iter()) {
            if *tag == OperandType::STRING {
                let start = (*payload >> 32) as usize;
                let end = (*payload & 0xFFFFFFFF) as usize;
                let new_start = new_string_pool.len();

                new_string_pool.extend_from_slice(&self.string_pool[start..end]);

                let new_end = new_string_pool.len();

                *payload = (new_start as u64) << 32 | (new_end as u64);
            }
        }

        new_string_pool.shrink_to_fit();

        self.string_pool = new_string_pool;
    }

    pub fn add_character(&mut self, character: char) -> u16 {
        let payload = character as u64;
        let hash = {
            let mut hasher = FxHasher::default();

            character.hash(&mut hasher);

            hasher.finish()
        };

        let (index, found) = self.payloads.insert_full(hash, payload);

        if found.is_none() {
            self.tags.push(OperandType::CHARACTER);
        }

        index as u16
    }

    pub fn get_character(&self, index: u16) -> Option<char> {
        let payload = *self.payloads.get_index(index as usize)?.1;

        if index < self.payloads.len() as u16 {
            std::char::from_u32(payload as u32)
        } else {
            None
        }
    }

    pub fn add_float(&mut self, float: f64) -> u16 {
        let payload = float.to_bits();
        let hash = {
            let mut hasher = FxHasher::default();

            payload.hash(&mut hasher);

            hasher.finish()
        };
        let (index, found) = self.payloads.insert_full(hash, payload);

        if found.is_none() {
            self.tags.push(OperandType::FLOAT);
        }

        index as u16
    }

    pub fn get_float(&self, index: u16) -> Option<f64> {
        let payload = *self.payloads.get_index(index as usize)?.1;

        if index < self.payloads.len() as u16 {
            Some(f64::from_bits(payload))
        } else {
            None
        }
    }

    pub fn add_integer(&mut self, integer: i64) -> u16 {
        let hash = {
            let mut hasher = FxHasher::default();

            integer.hash(&mut hasher);

            hasher.finish()
        };
        let payload = u64::from_le_bytes(integer.to_le_bytes());
        let (index, found) = self.payloads.insert_full(hash, payload);

        if found.is_none() {
            self.tags.push(OperandType::INTEGER);
        }

        index as u16
    }

    pub fn get_integer(&self, index: u16) -> Option<i64> {
        let payload = *self.payloads.get_index(index as usize)?.1;

        debug_assert!(self.tags[index as usize] == OperandType::INTEGER);

        if index < self.payloads.len() as u16 {
            Some(i64::from_le_bytes(payload.to_le_bytes()))
        } else {
            None
        }
    }

    pub fn add_array(&mut self, members: &[Address], array_type: OperandType) -> u16 {
        let start = self.array_pool.len();

        self.array_pool.extend_from_slice(members);

        let end = self.array_pool.len() as u32;
        let payload = (start as u64) << 32 | (end as u64);
        let index = self.payloads.len() as u16;

        let hash = {
            let mut hasher = FxHasher::default();

            members.hash(&mut hasher);

            hasher.finish()
        };

        self.payloads.insert(hash, payload);
        self.tags.push(array_type);

        index
    }

    pub fn get_array(&self, index: u16) -> Option<&[Address]> {
        let payload = *self.payloads.get_index(index as usize)?.1;
        let start = (payload >> 32) as usize;
        let end = (payload & 0xFFFFFFFF) as usize;

        if start < end && end <= self.array_pool.len() {
            Some(&self.array_pool[start..end])
        } else {
            None
        }
    }

    pub fn add_string(&mut self, bytes: &[u8]) -> u16 {
        let hash = {
            let mut hasher = FxHasher::default();

            bytes.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(existing_index) = self.payloads.get_index_of(&hash) {
            existing_index as u16
        } else {
            let start = self.string_pool.len();

            self.string_pool.extend_from_slice(bytes);

            let end = self.string_pool.len();
            let payload = (start as u64) << 32 | (end as u64);
            let index = self.payloads.len() as u16;

            self.payloads.insert(hash, payload);
            self.tags.push(OperandType::STRING);

            index
        }
    }

    pub fn get_string(&self, index: u16) -> Option<&str> {
        let payload = *self.payloads.get_index(index as usize)?.1;
        let start = (payload >> 32) as usize;
        let end = (payload & 0xFFFFFFFF) as usize;

        if start <= end && end <= self.string_pool.len() {
            Some(self.get_string_pool_range(start..end))
        } else {
            None
        }
    }

    pub fn get_string_raw_parts(&self, index: u16) -> Option<(*const u8, usize)> {
        let payload = *self.payloads.get_index(index as usize)?.1;
        let start = (payload >> 32) as usize;
        let end = (payload & 0xFFFFFFFF) as usize;
        let string = self.get_string_pool_range(start..end);

        if start <= end && end <= self.string_pool.len() {
            Some((string.as_ptr(), string.len()))
        } else {
            None
        }
    }

    pub fn push_str_to_string_pool(&mut self, bytes: &[u8]) -> (u32, u32) {
        let hash = {
            let mut hasher = FxHasher::default();

            bytes.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(existing_index) = self.payloads.get_index_of(&hash) {
            let payload = self.payloads[existing_index];
            let start = (payload >> 32) as u32;
            let end = (payload & 0xFFFFFFFF) as u32;

            (start, end)
        } else {
            let start = self.string_pool.len() as u32;

            self.string_pool.extend_from_slice(bytes);

            let end = self.string_pool.len() as u32;

            (start, end)
        }
    }

    pub fn add_pooled_string(&mut self, start: u32, end: u32) -> u16 {
        let string = self.get_string_pool_range(start as usize..end as usize);
        let hash = {
            let mut hasher = FxHasher::default();

            string.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(existing_index) = self.payloads.get_index_of(&hash) {
            existing_index as u16
        } else {
            let payload = (start as u64) << 32 | (end as u64);

            let (index, _) = self.payloads.insert_full(hash, payload);
            self.tags.push(OperandType::STRING);

            index as u16
        }
    }
}

pub struct ConstantTableIterator<'a> {
    table: &'a ConstantTable,
    index: usize,
}

impl Iterator for ConstantTableIterator<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.table.payloads.len() {
            return None;
        }

        let tag = self.table.tags[self.index];
        let payload = self.table.payloads[self.index];
        let string = match tag {
            OperandType::CHARACTER => char::from_u32(payload as u32)?.to_string(),
            OperandType::FLOAT => f64::from_bits(payload).to_string(),
            OperandType::INTEGER => (payload as i64).to_string(),
            OperandType::STRING => {
                let payload = *self.table.payloads.get_index(self.index)?.1;
                let start = (payload >> 32) as usize;
                let end = (payload & 0xFFFFFFFF) as usize;
                let slice = self.table.get_string_pool_range(start..end);

                String::from(slice)
            }
            OperandType::ARRAY_INTEGER => {
                let payload = *self.table.payloads.get_index(self.index)?.1;
                let start = (payload >> 32) as usize;
                let end = (payload & 0xFFFFFFFF) as usize;
                let addresses = &self.table.array_pool[start..end];

                let mut string = String::from("[");

                for (i, address) in addresses.iter().enumerate() {
                    if i > 0 {
                        string.push_str(", ");
                    }

                    string.push_str(&address.to_string(OperandType::INTEGER));
                }

                string.push(']');

                string
            }
            _ => todo!(),
        };

        self.index += 1;

        Some(string)
    }
}

impl Eq for ConstantTable {}

impl PartialEq for ConstantTable {
    fn eq(&self, other: &Self) -> bool {
        self.payloads == other.payloads
            && self.tags == other.tags
            && self.string_pool == other.string_pool
    }
}
