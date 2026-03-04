use std::{
    hash::{Hash, Hasher},
    ops::Range,
};

use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHasher};
use serde::{Deserialize, Serialize};

use crate::small_type::SmallType;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConstantTable {
    payloads: IndexMap<ConstantKey, u64, FxBuildHasher>,
    tags: Vec<SmallType>,
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

    pub fn add_character(&mut self, character: char) -> ConstantId {
        let payload = character as u64;
        let key = ConstantKey::from_payload_and_tag(payload, SmallType::CHARACTER);
        let (index, found) = self.payloads.insert_full(key, payload);

        if found.is_none() {
            self.tags.push(SmallType::CHARACTER);
        }

        ConstantId(index as u16)
    }

    pub fn get_character(&self, id: ConstantId) -> Option<char> {
        let index = id.0 as usize;
        let payload = *self.payloads.get_index(index)?.1;

        char::from_u32(payload as u32)
    }

    pub fn add_u32(&mut self, integer: u32) -> ConstantId {
        let payload = integer as u64;
        let key = ConstantKey::from_payload_and_tag(payload, SmallType::U_32);
        let (index, found) = self.payloads.insert_full(key, payload);

        if found.is_none() {
            self.tags.push(SmallType::U_32);
        }

        ConstantId(index as u16)
    }

    pub fn get_u32(&self, id: ConstantId) -> Option<u32> {
        let index = id.0 as usize;
        let payload = *self.payloads.get_index(index)?.1;

        Some(payload as u32)
    }

    pub fn add_i32(&mut self, integer: i32) -> ConstantId {
        let bytes = integer.to_le_bytes();
        let payload = u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], 0, 0, 0, 0]);
        let key = ConstantKey::from_payload_and_tag(payload, SmallType::I_32);
        let (index, found) = self.payloads.insert_full(key, payload);

        if found.is_none() {
            self.tags.push(SmallType::I_32);
        }

        ConstantId(index as u16)
    }

    pub fn get_i32(&self, id: ConstantId) -> Option<i32> {
        let index = id.0 as usize;
        let payload = *self.payloads.get_index(index)?.1;
        let bytes = payload.to_le_bytes();

        Some(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn add_u64(&mut self, integer: u64) -> ConstantId {
        let payload = integer;
        let key = ConstantKey::from_payload_and_tag(payload, SmallType::U_64);
        let (index, found) = self.payloads.insert_full(key, payload);

        if found.is_none() {
            self.tags.push(SmallType::U_64);
        }

        ConstantId(index as u16)
    }

    pub fn get_u64(&self, id: ConstantId) -> Option<u64> {
        let index = id.0 as usize;
        let payload = *self.payloads.get_index(index)?.1;

        Some(payload)
    }

    pub fn add_i64(&mut self, integer: i64) -> ConstantId {
        let payload = u64::from_le_bytes(integer.to_le_bytes());
        let key = ConstantKey::from_payload_and_tag(payload, SmallType::I_64);
        let (index, found) = self.payloads.insert_full(key, payload);

        if found.is_none() {
            self.tags.push(SmallType::I_64);
        }

        ConstantId(index as u16)
    }

    pub fn get_i64(&self, id: ConstantId) -> Option<i64> {
        let index = id.0 as usize;
        let payload = *self.payloads.get_index(index)?.1;

        Some(i64::from_le_bytes(payload.to_le_bytes()))
    }

    pub fn add_u128(&mut self, integer: u128) -> (ConstantId, ConstantId) {
        let bytes = integer.to_le_bytes();
        let left_payload = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        let right_payload = u64::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        let left_id = {
            let key = ConstantKey::from_payload_and_tag(left_payload, SmallType::U_128);
            let (index, found) = self.payloads.insert_full(key, left_payload);

            if found.is_none() {
                self.tags.push(SmallType::U_128);
            }

            ConstantId(index as u16)
        };
        let right_id = {
            let key = ConstantKey::from_payload_and_tag(right_payload, SmallType::U_128);
            let (index, found) = self.payloads.insert_full(key, right_payload);

            if found.is_none() {
                self.tags.push(SmallType::U_128);
            }

            ConstantId(index as u16)
        };

        (left_id, right_id)
    }

    pub fn get_u128(&self, left_id: ConstantId, right_id: ConstantId) -> Option<u128> {
        let left_index = left_id.0 as usize;
        let right_index = right_id.0 as usize;
        let left_payload = *self.payloads.get_index(left_index)?.1;
        let right_payload = *self.payloads.get_index(right_index)?.1;

        let mut bytes = [0u8; 16];

        bytes[0..8].copy_from_slice(&left_payload.to_le_bytes());
        bytes[8..16].copy_from_slice(&right_payload.to_le_bytes());

        Some(u128::from_le_bytes(bytes))
    }

    pub fn add_i128(&mut self, integer: i128) -> (ConstantId, ConstantId) {
        let bytes = integer.to_le_bytes();
        let left_payload = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        let right_payload = u64::from_le_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        let left_id = {
            let key = ConstantKey::from_payload_and_tag(left_payload, SmallType::I_128);
            let (index, found) = self.payloads.insert_full(key, left_payload);

            if found.is_none() {
                self.tags.push(SmallType::I_128);
            }

            ConstantId(index as u16)
        };
        let right_id = {
            let key = ConstantKey::from_payload_and_tag(right_payload, SmallType::I_128);
            let (index, found) = self.payloads.insert_full(key, right_payload);

            if found.is_none() {
                self.tags.push(SmallType::I_128);
            }

            ConstantId(index as u16)
        };

        (left_id, right_id)
    }

    pub fn get_i128(&self, left_id: ConstantId, right_id: ConstantId) -> Option<i128> {
        let left_index = left_id.0 as usize;
        let right_index = right_id.0 as usize;
        let left_payload = *self.payloads.get_index(left_index)?.1;
        let right_payload = *self.payloads.get_index(right_index)?.1;

        let mut bytes = [0u8; 16];

        bytes[0..8].copy_from_slice(&left_payload.to_le_bytes());
        bytes[8..16].copy_from_slice(&right_payload.to_le_bytes());

        Some(i128::from_le_bytes(bytes))
    }

    pub fn add_f32(&mut self, float: f32) -> ConstantId {
        let payload = float.to_bits() as u64;
        let key = ConstantKey::from_payload_and_tag(payload, SmallType::F_32);
        let (index, found) = self.payloads.insert_full(key, payload);

        if found.is_none() {
            self.tags.push(SmallType::F_32);
        }

        ConstantId(index as u16)
    }

    pub fn get_f32(&self, id: ConstantId) -> Option<f32> {
        let index = id.0 as usize;
        let payload = *self.payloads.get_index(index)?.1;

        Some(f32::from_bits(payload as u32))
    }

    pub fn add_f64(&mut self, float: f64) -> ConstantId {
        let payload = float.to_bits();
        let key = ConstantKey::from_payload_and_tag(payload, SmallType::F_64);
        let (index, found) = self.payloads.insert_full(key, payload);

        if found.is_none() {
            self.tags.push(SmallType::F_64);
        }

        ConstantId(index as u16)
    }

    pub fn get_f64(&self, id: ConstantId) -> Option<f64> {
        let index = id.0 as usize;
        let payload = *self.payloads.get_index(index)?.1;

        Some(f64::from_bits(payload))
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

            self.tags.push(SmallType::STRING);

            ConstantId(index as u16)
        }
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
        let key = ConstantKey::from_payload_and_tag(payload, SmallType::STRING);

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
            self.tags.push(SmallType::STRING);

            ConstantId(index as u16)
        }
    }

    pub fn display_iter<'a>(&'a self) -> ConstantTableDisplayIterator<'a> {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
struct ConstantKey(u64);

impl ConstantKey {
    pub fn from_payload_and_tag(payload: u64, tag: SmallType) -> Self {
        let mut hasher = FxHasher::default();

        payload.hash(&mut hasher);
        tag.hash(&mut hasher);

        Self(hasher.finish())
    }

    pub fn from_str(str: &str) -> Self {
        let mut hasher = FxHasher::default();

        str.hash(&mut hasher);
        SmallType::STRING.hash(&mut hasher);

        Self(hasher.finish())
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
            SmallType::CHARACTER => char::from_u32(payload as u32)?.to_string(),
            SmallType::F_64 => f64::from_bits(payload).to_string(),
            SmallType::I_64 => (payload as i64).to_string(),
            SmallType::STRING => {
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
        let charcter_id = table.add_character('a');
        let retrieved_character = table.get_character(charcter_id).unwrap();

        assert_eq!(retrieved_character, 'a');
    }

    #[test]
    fn float() {
        let mut table = ConstantTable::new();
        let float_id = table.add_f64(42.0);
        let retrieved_float = table.get_f64(float_id).unwrap();

        assert_eq!(retrieved_float, 42.0);
    }

    #[test]
    fn integer() {
        let mut table = ConstantTable::new();
        let integer_id = table.add_i64(42);
        let retrieved_integer = table.get_i64(integer_id).unwrap();

        assert_eq!(retrieved_integer, 42);
    }

    #[test]
    fn string() {
        let mut table = ConstantTable::new();
        let string_id = table.add_string("foobar");
        let retrieved_string = table.get_string(string_id).unwrap();

        assert_eq!(retrieved_string, "foobar");
    }
}
