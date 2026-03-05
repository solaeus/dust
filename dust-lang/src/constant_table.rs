use std::{
    hash::{Hash, Hasher},
    ops::Range,
};

use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHasher};
use serde::{Deserialize, Serialize};

use crate::{dust_error::InternalError, instruction::SmallType};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConstantTable {
    bytes: Vec<u8>,
    byte_indices: IndexMap<ConstantKey, u32, FxBuildHasher>,
    tags: Vec<SmallType>,
    string_pool: String,
}

impl ConstantTable {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            byte_indices: IndexMap::default(),
            tags: Vec::new(),
            string_pool: String::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.byte_indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.byte_indices.is_empty()
    }

    pub fn get_string_pool_range(&self, range: Range<usize>) -> &str {
        self.string_pool.get(range).unwrap_or_default()
    }

    pub fn add_character(&mut self, character: char) -> ConstantId {
        let bytes = (character as u32).to_le_bytes();

        self.add_bytes(&bytes, SmallType::CHARACTER)
    }

    pub fn get_character(&self, id: ConstantId) -> Result<char, InternalError> {
        let start = *self
            .byte_indices
            .get_index(id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?
            .1 as usize;
        let end = start + 4;
        let bytes = self
            .bytes
            .get(start..end)
            .ok_or(InternalError::InvalidConstantTable)?;

        char::from_u32(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            .ok_or(InternalError::InvalidConstantTable)
    }

    pub fn add_u32(&mut self, integer: u32) -> ConstantId {
        let bytes = integer.to_le_bytes();

        self.add_bytes(&bytes, SmallType::U_32)
    }

    pub fn get_u32(&self, id: ConstantId) -> Result<u32, InternalError> {
        let start = *self
            .byte_indices
            .get_index(id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?
            .1 as usize;
        let end = start + 4;
        let bytes = self
            .bytes
            .get(start..end)
            .ok_or(InternalError::InvalidConstantTable)?;

        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn add_i32(&mut self, integer: i32) -> ConstantId {
        let bytes = integer.to_le_bytes();

        self.add_bytes(&bytes, SmallType::I_32)
    }

    pub fn get_i32(&self, id: ConstantId) -> Result<i32, InternalError> {
        let start = *self
            .byte_indices
            .get_index(id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?
            .1 as usize;
        let end = start + 4;
        let bytes = self
            .bytes
            .get(start..end)
            .ok_or(InternalError::InvalidConstantTable)?;

        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn add_u64(&mut self, integer: u64) -> ConstantId {
        let bytes = integer.to_le_bytes();

        self.add_bytes(&bytes, SmallType::U_64)
    }

    pub fn get_u64(&self, id: ConstantId) -> Result<u64, InternalError> {
        let start = *self
            .byte_indices
            .get_index(id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?
            .1 as usize;
        let end = start + 8;
        let bytes = self
            .bytes
            .get(start..end)
            .ok_or(InternalError::InvalidConstantTable)?;

        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    pub fn add_i64(&mut self, integer: i64) -> ConstantId {
        let bytes = integer.to_le_bytes();

        self.add_bytes(&bytes, SmallType::I_64)
    }

    pub fn get_i64(&self, id: ConstantId) -> Result<i64, InternalError> {
        let start = *self
            .byte_indices
            .get_index(id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?
            .1 as usize;
        let end = start + 8;
        let bytes = self
            .bytes
            .get(start..end)
            .ok_or(InternalError::InvalidConstantTable)?;

        Ok(i64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    pub fn add_u128(&mut self, integer: u128) -> ConstantId {
        let bytes = integer.to_le_bytes();

        self.add_bytes(&bytes, SmallType::U_128)
    }

    pub fn get_u128(&self, left_id: ConstantId) -> Result<u128, InternalError> {
        let start = *self
            .byte_indices
            .get_index(left_id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?
            .1 as usize;
        let end = start + 16;
        let bytes = self
            .bytes
            .get(start..end)
            .ok_or(InternalError::InvalidConstantTable)?;

        Ok(u128::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]))
    }

    pub fn add_i128(&mut self, integer: i128) -> ConstantId {
        let bytes = integer.to_le_bytes();

        self.add_bytes(&bytes, SmallType::I_128)
    }

    pub fn get_i128(&self, left_id: ConstantId) -> Result<i128, InternalError> {
        let start = *self
            .byte_indices
            .get_index(left_id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?
            .1 as usize;
        let end = start + 16;
        let bytes = self
            .bytes
            .get(start..end)
            .ok_or(InternalError::InvalidConstantTable)?;

        Ok(i128::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]))
    }

    pub fn add_f32(&mut self, float: f32) -> ConstantId {
        let bytes = float.to_bits().to_le_bytes();

        self.add_bytes(&bytes, SmallType::F_32)
    }

    pub fn get_f32(&self, id: ConstantId) -> Result<f32, InternalError> {
        let start = *self
            .byte_indices
            .get_index(id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?
            .1 as usize;
        let end = start + 4;
        let bytes = self
            .bytes
            .get(start..end)
            .ok_or(InternalError::InvalidConstantTable)?;

        Ok(f32::from_bits(u32::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
        ])))
    }

    pub fn add_f64(&mut self, float: f64) -> ConstantId {
        let bytes = float.to_bits().to_le_bytes();

        self.add_bytes(&bytes, SmallType::F_64)
    }

    pub fn get_f64(&self, id: ConstantId) -> Result<f64, InternalError> {
        let start = *self
            .byte_indices
            .get_index(id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?
            .1 as usize;
        let end = start + 8;
        let bytes = self
            .bytes
            .get(start..end)
            .ok_or(InternalError::InvalidConstantTable)?;

        Ok(f64::from_bits(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])))
    }

    pub fn add_string(&mut self, str: &str) -> ConstantId {
        let str_bytes = str.as_bytes();
        let key = ConstantKey::new(str_bytes, SmallType::STRING);

        if let Some(existing) = self.byte_indices.get_index_of(&key) {
            ConstantId(existing as u16)
        } else {
            let start = self.string_pool.len();
            let end = self.string_pool.len() + str.len();
            let encoded_range = (start as u64) << 32 | (end as u64);
            let range_bytes = encoded_range.to_le_bytes();

            let next_byte = self.bytes.len() as u32;
            let index = self.byte_indices.len();

            self.byte_indices.insert(key, next_byte);
            self.bytes.extend_from_slice(&range_bytes);
            self.string_pool.push_str(str);

            ConstantId(index as u16)
        }
    }

    pub fn get_string(&self, id: ConstantId) -> Option<&str> {
        let start = *self.byte_indices.get_index(id.0 as usize)?.1 as usize;
        let end = start + 8;
        let bytes = self.bytes.get(start..end)?;
        let encoded_range = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        let str_start = (encoded_range >> 32) as usize;
        let str_end = (encoded_range & 0xFFFFFFFF) as usize;

        self.string_pool.get(str_start..str_end)
    }

    pub fn get_string_raw_parts(&self, id: ConstantId) -> Option<(*const u8, usize)> {
        self.get_string(id).map(|str| (str.as_ptr(), str.len()))
    }

    pub fn push_str_to_string_pool(&mut self, str: &str) -> Result<(u32, u32), InternalError> {
        let bytes = str.as_bytes();
        let key = ConstantKey::new(bytes, SmallType::STRING);

        if let Some(str_start) = self.byte_indices.get(&key) {
            let str_end = str_start + 8;
            let bytes = self
                .bytes
                .get(*str_start as usize..str_end as usize)
                .ok_or(InternalError::InvalidConstantTable)?;
            let encoded_range = u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]);
            let str_start = (encoded_range >> 32) as u32;
            let str_end = (encoded_range & 0xFFFFFFFF) as u32;

            Ok((str_start, str_end))
        } else {
            let start = self.string_pool.len() as u32;
            let end = start + str.len() as u32;
            let encoded_range = (start as u64) << 32 | (end as u64);
            let bytes = encoded_range.to_le_bytes();

            self.string_pool.push_str(str);
            self.add_bytes(&bytes, SmallType::STRING);

            Ok((start, end))
        }
    }

    pub fn add_pooled_string(&mut self, start: u32, end: u32) -> ConstantId {
        let bytes = self
            .get_string_pool_range(start as usize..end as usize)
            .as_bytes();
        let key = ConstantKey::new(bytes, SmallType::STRING);

        if let Some(existing) = self.byte_indices.get_index_of(&key) {
            ConstantId(existing as u16)
        } else {
            let encoded_range = (start as u64) << 32 | (end as u64);
            let bytes = encoded_range.to_le_bytes();

            self.add_bytes(&bytes, SmallType::STRING)
        }
    }

    pub fn display_iter<'a>(&'a self) -> ConstantTableDisplayIterator<'a> {
        ConstantTableDisplayIterator {
            table: self,
            index: 0,
        }
    }

    fn add_bytes(&mut self, bytes: &[u8], tag: SmallType) -> ConstantId {
        let key = ConstantKey::new(bytes, tag);
        let next_byte = self.bytes.len() as u32;
        let (index, found) = self.byte_indices.insert_full(key, next_byte);

        if found.is_none() {
            self.bytes.extend_from_slice(bytes);
            self.tags.push(tag);
        }

        ConstantId(index as u16)
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
    pub fn new(bytes: &[u8], tag: SmallType) -> Self {
        let mut hasher = FxHasher::default();

        tag.hash(&mut hasher);

        for byte in bytes {
            hasher.write_u8(*byte);
        }

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
        if self.index >= self.table.len() {
            return None;
        }

        let tag = self.table.tags[self.index];
        let id = ConstantId(self.index as u16);
        let value_string = match tag {
            SmallType::CHARACTER => self.table.get_character(id).unwrap_or_default().to_string(),
            SmallType::U_32 => self.table.get_u32(id).unwrap_or_default().to_string(),
            SmallType::I_32 => self.table.get_i32(id).unwrap_or_default().to_string(),
            SmallType::U_64 => self.table.get_u64(id).unwrap_or_default().to_string(),
            SmallType::I_64 => self.table.get_i64(id).unwrap_or_default().to_string(),
            SmallType::U_128 => self.table.get_u128(id).unwrap_or_default().to_string(),
            SmallType::I_128 => self.table.get_i128(id).unwrap_or_default().to_string(),
            SmallType::F_32 => self.table.get_f32(id).unwrap_or_default().to_string(),
            SmallType::F_64 => self.table.get_f64(id).unwrap_or_default().to_string(),
            SmallType::STRING => self.table.get_string(id).unwrap_or_default().to_string(),
            _ => "Unknown constant type".to_string(),
        };
        let type_string = tag.to_string();

        self.index += 1;

        Some((value_string, type_string))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_table() -> ConstantTable {
        let mut table = ConstantTable::new();

        table.add_character('q');
        table.add_u32(666);
        table.add_i32(-666);
        table.add_u64(666);
        table.add_i64(666);
        table.add_u128(666);
        table.add_i128(666);
        table.add_f32(666.0);
        table.add_f64(666.0);
        table.add_string("666");

        table
    }

    #[test]
    fn interns() {
        let mut table = ConstantTable::new();

        let first_id = table.add_character('a');
        let second_id = table.add_character('a');

        assert_eq!(first_id, second_id);
        assert_eq!(table.len(), 1);

        let first_id = table.add_u32(42);
        let second_id = table.add_u32(42);

        assert_eq!(first_id, second_id);
        assert_eq!(table.len(), 2);

        let first_id = table.add_i32(-42);
        let second_id = table.add_i32(-42);

        assert_eq!(first_id, second_id);
        assert_eq!(table.len(), 3);

        let first_id = table.add_u64(42);
        let second_id = table.add_u64(42);

        assert_eq!(first_id, second_id);
        assert_eq!(table.len(), 4);

        let first_id = table.add_i64(42);
        let second_id = table.add_i64(42);

        assert_eq!(first_id, second_id);
        assert_eq!(table.len(), 5);

        let first_id = table.add_u128(42);
        let second_id = table.add_u128(42);

        assert_eq!(first_id, second_id);
        assert_eq!(table.len(), 6);

        let first_id = table.add_i128(42);
        let second_id = table.add_i128(42);

        assert_eq!(first_id, second_id);
        assert_eq!(table.len(), 7);

        let first_id = table.add_string("foobar");
        let second_id = table.add_string("foobar");

        assert_eq!(first_id, second_id);
        assert_eq!(table.len(), 8);
    }

    #[test]
    fn character() {
        let mut table = create_test_table();
        let charcter_id = table.add_character('a');
        let retrieved_character = table.get_character(charcter_id).unwrap();

        assert_eq!(retrieved_character, 'a');
    }

    #[test]
    fn u32() {
        let mut table = create_test_table();
        let integer_id = table.add_u32(42);
        let retrieved_integer = table.get_u32(integer_id).unwrap();

        assert_eq!(retrieved_integer, 42);
    }

    #[test]
    fn i32() {
        let mut table = create_test_table();
        let integer_id = table.add_i32(-42);
        let retrieved_integer = table.get_i32(integer_id).unwrap();

        assert_eq!(retrieved_integer, -42);
    }

    #[test]
    fn u64() {
        let mut table = create_test_table();
        let integer_id = table.add_u64(42);
        let retrieved_integer = table.get_u64(integer_id).unwrap();

        assert_eq!(retrieved_integer, 42);
    }

    #[test]
    fn i64() {
        let mut table = create_test_table();
        let integer_id = table.add_i64(42);
        let retrieved_integer = table.get_i64(integer_id).unwrap();

        assert_eq!(retrieved_integer, 42);
    }

    #[test]
    fn u128() {
        let mut table = create_test_table();
        let integer_id = table.add_u128(42);
        let retrieved_integer = table.get_u128(integer_id).unwrap();

        assert_eq!(retrieved_integer, 42);
    }

    #[test]
    fn i128() {
        let mut table = create_test_table();
        let integer_id = table.add_i128(42);
        let retrieved_integer = table.get_i128(integer_id).unwrap();

        assert_eq!(retrieved_integer, 42);
    }

    #[test]
    fn f32() {
        let mut table = create_test_table();
        let float_id = table.add_f32(42.0);
        let retrieved_float = table.get_f32(float_id).unwrap();

        assert_eq!(retrieved_float, 42.0);
    }

    #[test]
    fn f64() {
        let mut table = create_test_table();
        let float_id = table.add_f64(42.0);
        let retrieved_float = table.get_f64(float_id).unwrap();

        assert_eq!(retrieved_float, 42.0);
    }

    #[test]
    fn string() {
        let mut table = create_test_table();
        let string_id = table.add_string("foobar");
        let retrieved_string = table.get_string(string_id).unwrap();

        assert_eq!(retrieved_string, "foobar");
    }
}
