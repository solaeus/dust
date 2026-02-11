use std::{
    hash::{Hash, Hasher},
    ops::Range,
};

use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHasher};

use crate::instruction::OperandType;

#[derive(Debug, Default)]
pub struct ConstantTable {
    payloads: IndexMap<ConstantKey, u64, FxBuildHasher>,
    tags: Vec<OperandType>,
    string_pool: String,
}

impl ConstantTable {
    pub fn new() -> Self {
        Self {
            payloads: IndexMap::default(),
            tags: Vec::new(),
            string_pool: String::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.payloads.len()
    }

    pub fn is_empty(&self) -> bool {
        self.payloads.is_empty()
    }

    pub fn get_string_pool_range(&self, range: Range<usize>) -> &str {
        self.string_pool.get(range).unwrap_or_default()
    }

    pub fn finalize_string_pool(&mut self) {
        let mut new_string_pool = String::with_capacity(self.string_pool.len());

        for (payload, tag) in self.payloads.values_mut().zip(self.tags.iter()) {
            if *tag == OperandType::STRING {
                let start = (*payload >> 32) as usize;
                let end = (*payload & 0xFFFFFFFF) as usize;
                let new_start = new_string_pool.len();

                new_string_pool.push_str(&self.string_pool[start..end]);

                let new_end = new_string_pool.len();

                *payload = (new_start as u64) << 32 | (new_end as u64);
            }
        }

        new_string_pool.shrink_to_fit();

        self.string_pool = new_string_pool;
    }

    pub fn add_character(&mut self, character: char) -> ConstantId {
        let payload = character as u64;
        let key = ConstantKey::from_payload_and_tag(payload, OperandType::CHARACTER);
        let (index, found) = self.payloads.insert_full(key, payload);

        if found.is_none() {
            self.tags.push(OperandType::CHARACTER);
        }

        ConstantId(index as u16)
    }

    pub fn get_character(&self, id: ConstantId) -> Option<char> {
        let index = id.0 as usize;
        let payload = *self.payloads.get_index(index)?.1;

        if index < self.payloads.len() {
            std::char::from_u32(payload as u32)
        } else {
            None
        }
    }

    pub fn add_float(&mut self, float: f64) -> ConstantId {
        let payload = float.to_bits();
        let key = ConstantKey::from_payload_and_tag(payload, OperandType::FLOAT);
        let (index, found) = self.payloads.insert_full(key, payload);

        if found.is_none() {
            self.tags.push(OperandType::FLOAT);
        }

        ConstantId(index as u16)
    }

    pub fn get_float(&self, id: ConstantId) -> Option<f64> {
        let index = id.0 as usize;
        let payload = *self.payloads.get_index(index)?.1;

        if index < self.payloads.len() {
            Some(f64::from_bits(payload))
        } else {
            None
        }
    }

    pub fn add_integer(&mut self, integer: i64) -> ConstantId {
        let payload = u64::from_le_bytes(integer.to_le_bytes());
        let key = ConstantKey::from_payload_and_tag(payload, OperandType::INTEGER);
        let (index, found) = self.payloads.insert_full(key, payload);

        if found.is_none() {
            self.tags.push(OperandType::INTEGER);
        }

        ConstantId(index as u16)
    }

    pub fn get_integer(&self, id: ConstantId) -> Option<i64> {
        let index = id.0 as usize;
        let payload = *self.payloads.get_index(index)?.1;

        if index < self.payloads.len() {
            Some(i64::from_le_bytes(payload.to_le_bytes()))
        } else {
            None
        }
    }

    pub fn add_string(&mut self, str: &str) -> ConstantId {
        let key = ConstantKey::from_str(str);

        if let Some(existing_index) = self.payloads.get_index_of(&key) {
            ConstantId(existing_index as u16)
        } else {
            let start = self.string_pool.len();
            let end = self.string_pool.len() + str.len();
            let payload = (start as u64) << 32 | (end as u64);

            self.string_pool.push_str(str);

            let (index, _) = self.payloads.insert_full(key, payload);

            self.tags.push(OperandType::STRING);

            ConstantId(index as u16)
        }
    }

    pub fn get_string_from_key(&self, key: &ConstantKey) -> Option<&str> {
        if let Some(index) = self.payloads.get_index_of(key) {
            let constant_id = ConstantId(index as u16);

            if let Some(found) = self.get_string(constant_id) {
                return Some(found);
            }
        }

        None
    }

    pub fn get_string(&self, id: ConstantId) -> Option<&str> {
        let index = id.0 as usize;
        let payload = *self.payloads.get_index(index)?.1;
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

    pub fn push_str_to_string_pool(&mut self, str: &str) -> (u32, u32) {
        let start = self.string_pool.len();
        let end = self.string_pool.len() + str.len();
        let payload = (start as u64) << 32 | (end as u64);
        let key = ConstantKey::from_payload_and_tag(payload, OperandType::STRING);

        if let Some(existing_index) = self.payloads.get_index_of(&key) {
            let payload = self.payloads[existing_index];
            let start = (payload >> 32) as u32;
            let end = (payload & 0xFFFFFFFF) as u32;

            (start, end)
        } else {
            let start = self.string_pool.len() as u32;

            self.string_pool.push_str(str);

            let end = self.string_pool.len() as u32;

            (start, end)
        }
    }

    pub fn add_pooled_string(&mut self, start: u32, end: u32) -> ConstantId {
        let str = self.get_string_pool_range(start as usize..end as usize);
        let key = ConstantKey::from_str(str);

        if let Some(existing_index) = self.payloads.get_index_of(&key) {
            ConstantId(existing_index as u16)
        } else {
            let payload = (start as u64) << 32 | (end as u64);

            let (index, _) = self.payloads.insert_full(key, payload);
            self.tags.push(OperandType::STRING);

            ConstantId(index as u16)
        }
    }

    pub fn display_iterator<'a>(&'a self) -> ConstantTableDisplayIterator<'a> {
        ConstantTableDisplayIterator {
            table: self,
            index: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstantId(pub(crate) u16);

impl ConstantId {
    pub fn inner(&self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum ConstantKey {
    Payload(u64, OperandType),
    Bytes(u64),
}

impl ConstantKey {
    pub fn from_payload_and_tag(payload: u64, tag: OperandType) -> Self {
        Self::Payload(payload, tag)
    }

    pub fn from_str(str: &str) -> Self {
        let mut hasher = FxHasher::default();

        str.hash(&mut hasher);

        Self::Bytes(hasher.finish())
    }
}

pub struct ConstantTableDisplayIterator<'a> {
    table: &'a ConstantTable,
    index: usize,
}

impl Iterator for ConstantTableDisplayIterator<'_> {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.table.payloads.len() {
            return None;
        }

        let tag = self.table.tags[self.index];
        let payload = self.table.payloads[self.index];
        let value_string = match tag {
            OperandType::CHARACTER => char::from_u32(payload as u32)?.to_string(),
            OperandType::FLOAT => f64::from_bits(payload).to_string(),
            OperandType::INTEGER => (payload as i64).to_string(),
            OperandType::STRING => {
                let payload = *self.table.payloads.get_index(self.index)?.1;
                let start = (payload >> 32) as usize;
                let end = (payload & 0xFFFFFFFF) as usize;

                self.table.get_string_pool_range(start..end).to_string()
            }
            _ => todo!(),
        };
        let type_string = tag.to_string();

        self.index += 1;

        Some((value_string, type_string))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character() {
        let mut table = ConstantTable::new();
        let charcter_index = table.add_character('a');
        let retrieved_character = table.get_character(charcter_index).unwrap();

        assert_eq!(retrieved_character, 'a');
    }

    #[test]
    fn float() {
        let mut table = ConstantTable::new();
        let float_index = table.add_float(42.0);
        let retrieved_float = table.get_float(float_index).unwrap();

        assert_eq!(retrieved_float, 42.0);
    }

    #[test]
    fn integer() {
        let mut table = ConstantTable::new();
        let integer_index = table.add_integer(42);
        let retrieved_integer = table.get_integer(integer_index).unwrap();

        assert_eq!(retrieved_integer, 42);
    }

    #[test]
    fn string() {
        let mut table = ConstantTable::new();
        let string_index = table.add_string("foobar");
        let retrieved_string = table.get_string(string_index).unwrap();

        assert_eq!(retrieved_string, "foobar");
    }
}
