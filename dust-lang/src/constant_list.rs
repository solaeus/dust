use std::{
    char,
    collections::HashMap,
    hash::{Hash, Hasher},
    ops::Range,
};

use rustc_hash::{FxBuildHasher, FxHasher};
use serde::{Deserialize, Serialize};

use crate::{dust_error::InternalError, instruction::OperandType};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConstantList {
    payloads: Vec<u64>,
    string_pool: String,
}

impl ConstantList {
    pub fn get_character(&self, id: ConstantId) -> Result<char, InternalError> {
        let payload = *self
            .payloads
            .get(id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?;

        char::from_u32(payload as u32).ok_or(InternalError::InvalidConstantTable)
    }

    pub fn get_u32(&self, id: ConstantId) -> Result<u32, InternalError> {
        let payload = *self
            .payloads
            .get(id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?;

        Ok(payload as u32)
    }

    pub fn get_i32(&self, id: ConstantId) -> Result<i32, InternalError> {
        let payload = *self
            .payloads
            .get(id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?;

        Ok(payload as i32)
    }

    pub fn get_u64(&self, id: ConstantId) -> Result<u64, InternalError> {
        let payload = *self
            .payloads
            .get(id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?;

        Ok(payload)
    }

    pub fn get_i64(&self, id: ConstantId) -> Result<i64, InternalError> {
        let payload = *self
            .payloads
            .get(id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?;

        Ok(payload as i64)
    }

    pub fn get_u128(&self, id: ConstantId) -> Result<u128, InternalError> {
        let low_payload = *self
            .payloads
            .get(id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?;
        let high_payload = *self
            .payloads
            .get((id.0 + 1) as usize)
            .ok_or(InternalError::InvalidConstantTable)?;
        let decoded = (high_payload as u128) << 64 | (low_payload as u128);

        Ok(decoded)
    }

    pub fn get_i128(&self, left_id: ConstantId) -> Result<i128, InternalError> {
        let low_payload = *self
            .payloads
            .get(left_id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?;
        let high_payload = *self
            .payloads
            .get((left_id.0 + 1) as usize)
            .ok_or(InternalError::InvalidConstantTable)?;
        let decoded = (high_payload as i128) << 64 | (low_payload as i128);

        Ok(decoded)
    }

    pub fn get_f32(&self, id: ConstantId) -> Result<f32, InternalError> {
        let payload = *self
            .payloads
            .get(id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?;

        Ok(f32::from_bits(payload as u32))
    }

    pub fn get_f64(&self, id: ConstantId) -> Result<f64, InternalError> {
        let payload = *self
            .payloads
            .get(id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?;

        Ok(f64::from_bits(payload))
    }

    pub fn get_string(&self, id: ConstantId) -> Result<&str, InternalError> {
        let payload = *self
            .payloads
            .get(id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?;
        let start = (payload >> 32) as usize;
        let end = (payload & 0xFFFFFFFF) as usize;

        self.string_pool
            .get(start..end)
            .ok_or(InternalError::InvalidConstantTable)
    }

    pub fn get_string_raw_parts(
        &self,
        id: ConstantId,
    ) -> Result<(*const u8, usize), InternalError> {
        self.get_string(id).map(|str| (str.as_ptr(), str.len()))
    }

    pub fn payloads(&self) -> &Vec<u64> {
        &self.payloads
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConstantListBuilder {
    payloads: Vec<u64>,
    tags: Vec<OperandType>,
    string_pool: String,
    interner: HashMap<(OperandType, u64), ConstantId, FxBuildHasher>,
}

impl ConstantListBuilder {
    pub fn new() -> Self {
        Self {
            payloads: Vec::new(),
            tags: Vec::new(),
            interner: HashMap::default(),
            string_pool: String::new(),
        }
    }

    pub fn build(self) -> (ConstantList, Vec<OperandType>) {
        (
            ConstantList {
                payloads: self.payloads,
                string_pool: self.string_pool,
            },
            self.tags,
        )
    }

    pub fn len(&self) -> usize {
        self.payloads.len()
    }

    pub fn is_empty(&self) -> bool {
        self.payloads.is_empty()
    }

    pub fn tag_count(&self) -> usize {
        self.tags.len()
    }

    pub fn get_string_pool_range(&self, range: Range<usize>) -> &str {
        self.string_pool.get(range).unwrap_or_default()
    }

    pub fn add_character(&mut self, character: char) -> ConstantId {
        let payload = character as u64;

        self.add_payload(payload, OperandType::CHARACTER)
    }

    pub fn get_character(&self, id: ConstantId) -> Result<char, InternalError> {
        let payload = *self
            .payloads
            .get(id.0 as usize)
            .ok_or(InternalError::InvalidConstantTable)?;

        char::from_u32(payload as u32).ok_or(InternalError::InvalidConstantTable)
    }

    pub fn add_u32(&mut self, integer: u32) -> ConstantId {
        let payload = integer as u64;

        self.add_payload(payload, OperandType::U_32)
    }

    pub fn add_i32(&mut self, integer: i32) -> ConstantId {
        let payload = integer as u64;

        self.add_payload(payload, OperandType::I_32)
    }

    pub fn add_u64(&mut self, integer: u64) -> ConstantId {
        self.add_payload(integer, OperandType::U_64)
    }

    pub fn add_i64(&mut self, integer: i64) -> ConstantId {
        let payload = integer as u64;

        self.add_payload(payload, OperandType::I_64)
    }

    pub fn add_u128(&mut self, integer: u128) -> ConstantId {
        let low_payload = (integer & 0xFFFFFFFFFFFFFFFF) as u64;
        let high_payload = (integer >> 64) as u64;

        let id = self.add_payload(low_payload, OperandType::U_128);
        self.add_payload(high_payload, OperandType::U_128);

        id
    }

    pub fn add_i128(&mut self, integer: i128) -> ConstantId {
        let low_payload = (integer & 0xFFFFFFFFFFFFFFFF) as u64;
        let high_payload = ((integer >> 64) & 0xFFFFFFFFFFFFFFFF) as u64;

        let id = self.add_payload(low_payload, OperandType::I_128);
        self.add_payload(high_payload, OperandType::I_128);

        id
    }

    pub fn add_f32(&mut self, float: f32) -> ConstantId {
        let payload = float.to_bits() as u64;

        self.add_payload(payload, OperandType::F_32)
    }

    pub fn add_f64(&mut self, float: f64) -> ConstantId {
        let payload = float.to_bits();

        self.add_payload(payload, OperandType::F_64)
    }

    pub fn add_string(&mut self, str: &str) -> ConstantId {
        let hash = {
            let mut hasher = FxHasher::default();

            str.hash(&mut hasher);

            hasher.finish()
        };
        let found = self.interner.get(&(OperandType::STRING, hash));

        if let Some(id) = found {
            return *id;
        }

        let start = self.string_pool.len() as u32;
        let end = start + str.len() as u32;
        let payload = (start as u64) << 32 | (end as u64);
        let id = ConstantId(self.payloads.len() as u16);

        self.string_pool.push_str(str);
        self.add_payload(payload, OperandType::STRING);
        self.interner.insert(
            (OperandType::STRING, hash),
            ConstantId(self.payloads.len() as u16 - 1),
        );

        id
    }

    fn add_payload(&mut self, payload: u64, tag: OperandType) -> ConstantId {
        let existing_id = self.interner.get(&(tag, payload));

        if let Some(id) = existing_id {
            return *id;
        }

        let id = ConstantId(self.payloads.len() as u16);

        self.payloads.push(payload);
        self.tags.push(tag);
        self.interner.insert((tag, payload), id);

        id
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ConstantId(u16);

impl ConstantId {
    pub fn inner(&self) -> u16 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_table(
        op: fn(&mut ConstantListBuilder) -> ConstantId,
    ) -> (ConstantList, ConstantId) {
        let mut table = ConstantListBuilder::new();

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

        let id = op(&mut table);

        (table.build().0, id)
    }

    #[test]
    fn interns() {
        let mut table = ConstantListBuilder::new();

        let first_id = table.add_character('a');
        let second_id = table.add_character('a');

        assert_eq!(first_id, second_id);

        let first_id = table.add_u32(42);
        let second_id = table.add_u32(42);

        assert_eq!(first_id, second_id);

        let first_id = table.add_i32(-42);
        let second_id = table.add_i32(-42);

        assert_eq!(first_id, second_id);

        let first_id = table.add_u64(42);
        let second_id = table.add_u64(42);

        assert_eq!(first_id, second_id);

        let first_id = table.add_i64(42);
        let second_id = table.add_i64(42);

        assert_eq!(first_id, second_id);

        let first_id = table.add_u128(42);
        let second_id = table.add_u128(42);

        assert_eq!(first_id, second_id);

        let first_id = table.add_i128(42);
        let second_id = table.add_i128(42);

        assert_eq!(first_id, second_id);

        let first_id = table.add_string("foobar");
        let second_id = table.add_string("foobar");

        assert_eq!(first_id, second_id);
    }

    #[test]
    fn character() {
        let (table, id) = create_test_table(|table| table.add_character('q'));
        let retrieved = table.get_character(id).unwrap();

        assert_eq!(retrieved, 'q');
    }

    #[test]
    fn u32() {
        let (table, id) = create_test_table(|table| table.add_u32(666));
        let retrieved = table.get_u32(id).unwrap();

        assert_eq!(retrieved, 666);
    }

    #[test]
    fn i32() {
        let (table, id) = create_test_table(|table| table.add_i32(-666));
        let retrieved = table.get_i32(id).unwrap();

        assert_eq!(retrieved, -666);
    }

    #[test]
    fn u64() {
        let (table, id) = create_test_table(|table| table.add_u64(666));
        let retrieved = table.get_u64(id).unwrap();

        assert_eq!(retrieved, 666);
    }

    #[test]
    fn i64() {
        let (table, id) = create_test_table(|table| table.add_i64(666));
        let retrieved = table.get_i64(id).unwrap();

        assert_eq!(retrieved, 666);
    }

    #[test]
    fn u128() {
        let (table, id) = create_test_table(|table| table.add_u128(666));
        let retrieved = table.get_u128(id).unwrap();

        assert_eq!(retrieved, 666);
    }

    #[test]
    fn i128() {
        let (table, id) = create_test_table(|table| table.add_i128(666));
        let retrieved = table.get_i128(id).unwrap();

        assert_eq!(retrieved, 666);
    }

    #[test]
    fn f32() {
        let (table, id) = create_test_table(|table| table.add_f32(666.0));
        let retrieved = table.get_f32(id).unwrap();

        assert_eq!(retrieved, 666.0);
    }

    #[test]
    fn f64() {
        let (table, id) = create_test_table(|table| table.add_f64(666.0));
        let retrieved = table.get_f64(id).unwrap();

        assert_eq!(retrieved, 666.0);
    }

    #[test]
    fn string() {
        let (table, id) = create_test_table(|table| table.add_string("666"));
        let retrieved = table.get_string(id).unwrap();

        assert_eq!(retrieved, "666");
    }
}
