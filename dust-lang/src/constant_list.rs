use std::{
    char,
    collections::HashMap,
    hash::{Hash, Hasher},
};

use annotate_snippets::Group;
use rustc_hash::{FxBuildHasher, FxHasher};
use serde::{Deserialize, Serialize};

use crate::{error::AnnotatedError, instruction::OperandType};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConstantList {
    payloads: Vec<u32>,
    string_pool: String,
}

impl ConstantList {
    pub fn get_u32(&self, id: ConstantId) -> Result<u32, ConstantListError> {
        let payload = *self
            .payloads
            .get(id.0 as usize)
            .ok_or(ConstantListError::MissingConstant(id))?;

        Ok(payload)
    }

    pub fn get_i32(&self, id: ConstantId) -> Result<i32, ConstantListError> {
        let payload = *self
            .payloads
            .get(id.0 as usize)
            .ok_or(ConstantListError::MissingConstant(id))?;

        Ok(payload as i32)
    }

    pub fn get_u64(&self, id: ConstantId) -> Result<u64, ConstantListError> {
        let payload_range = id.0 as usize..(id.0 + 2) as usize;
        let payloads = self
            .payloads
            .get(payload_range)
            .ok_or(ConstantListError::MissingConstant(id))?;
        let decoded = (payloads[1] as u64) << 32 | (payloads[0] as u64);

        Ok(decoded)
    }

    pub fn get_i64(&self, id: ConstantId) -> Result<i64, ConstantListError> {
        let payload_range = id.0 as usize..(id.0 + 2) as usize;
        let payloads = self
            .payloads
            .get(payload_range)
            .ok_or(ConstantListError::MissingConstant(id))?;
        let decoded = (payloads[1] as i64) << 32 | (payloads[0] as i64);

        Ok(decoded)
    }

    pub fn get_u128(&self, id: ConstantId) -> Result<u128, ConstantListError> {
        let payload_range = id.0 as usize..(id.0 + 4) as usize;
        let payloads = self
            .payloads
            .get(payload_range)
            .ok_or(ConstantListError::MissingConstant(id))?;
        let decoded = (payloads[3] as u128) << 96
            | (payloads[2] as u128) << 64
            | (payloads[1] as u128) << 32
            | (payloads[0] as u128);

        Ok(decoded)
    }

    pub fn get_i128(&self, id: ConstantId) -> Result<i128, ConstantListError> {
        let payload_range = id.0 as usize..(id.0 + 4) as usize;
        let payloads = self
            .payloads
            .get(payload_range)
            .ok_or(ConstantListError::MissingConstant(id))?;
        let decoded = (payloads[3] as i128) << 96
            | (payloads[2] as i128) << 64
            | (payloads[1] as i128) << 32
            | (payloads[0] as i128);

        Ok(decoded)
    }

    pub fn get_f32(&self, id: ConstantId) -> Result<f32, ConstantListError> {
        let payload = *self
            .payloads
            .get(id.0 as usize)
            .ok_or(ConstantListError::MissingConstant(id))?;

        Ok(f32::from_bits(payload))
    }

    pub fn get_f64(&self, id: ConstantId) -> Result<f64, ConstantListError> {
        let payload_range = id.0 as usize..(id.0 + 2) as usize;
        let payloads = self
            .payloads
            .get(payload_range)
            .ok_or(ConstantListError::MissingConstant(id))?;
        let payload = (payloads[1] as u64) << 32 | (payloads[0] as u64);

        Ok(f64::from_bits(payload))
    }

    pub fn get_character(&self, id: ConstantId) -> Result<char, ConstantListError> {
        let payload = *self
            .payloads
            .get(id.0 as usize)
            .ok_or(ConstantListError::MissingConstant(id))?;

        char::from_u32(payload).ok_or(ConstantListError::InvalidConstantPayload)
    }

    pub fn get_string(&self, id: ConstantId) -> Result<&str, ConstantListError> {
        let payload = *self
            .payloads
            .get(id.0 as usize)
            .ok_or(ConstantListError::MissingConstant(id))?;
        let start = (payload >> 16) as usize;
        let length = (payload & 0xFFFF) as usize;
        let end = start + length;

        self.string_pool
            .get(start..end)
            .ok_or(ConstantListError::InvalidConstantPayload)
    }

    pub fn get_string_raw_parts(
        &self,
        id: ConstantId,
    ) -> Result<(*const u8, usize), ConstantListError> {
        self.get_string(id).map(|str| (str.as_ptr(), str.len()))
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConstantListBuilder {
    payloads: Vec<u32>,
    tags: Vec<OperandType>,
    string_pool: String,
    interner: HashMap<u64, ConstantId, FxBuildHasher>,
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

    pub fn add_u32(&mut self, integer: u32) -> ConstantId {
        let key = {
            let mut hasher = FxHasher::default();

            OperandType::U_32.hash(&mut hasher);
            integer.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(id) = self.interner.get(&key) {
            return *id;
        }

        self.add_payloads([integer], key, OperandType::U_32)
    }

    pub fn add_i32(&mut self, integer: i32) -> ConstantId {
        let key = {
            let mut hasher = FxHasher::default();

            OperandType::I_32.hash(&mut hasher);
            integer.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(id) = self.interner.get(&key) {
            return *id;
        }

        self.add_payloads([integer as u32], key, OperandType::I_32)
    }

    pub fn add_u64(&mut self, integer: u64) -> ConstantId {
        let key = {
            let mut hasher = FxHasher::default();

            OperandType::U_64.hash(&mut hasher);
            integer.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(id) = self.interner.get(&key) {
            return *id;
        }

        let low_payload = (integer & 0xFFFFFFFF) as u32;
        let high_payload = ((integer >> 32) & 0xFFFFFFFF) as u32;

        self.add_payloads([low_payload, high_payload], key, OperandType::U_64)
    }

    pub fn add_i64(&mut self, integer: i64) -> ConstantId {
        let key = {
            let mut hasher = FxHasher::default();

            OperandType::I_64.hash(&mut hasher);
            integer.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(id) = self.interner.get(&key) {
            return *id;
        }

        let low_payload = (integer & 0xFFFFFFFF) as u32;
        let high_payload = ((integer >> 32) & 0xFFFFFFFF) as u32;

        self.add_payloads([low_payload, high_payload], key, OperandType::I_64)
    }

    pub fn add_u128(&mut self, integer: u128) -> ConstantId {
        let key = {
            let mut hasher = FxHasher::default();

            OperandType::U_128.hash(&mut hasher);
            integer.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(id) = self.interner.get(&key) {
            return *id;
        }

        let payload_0 = (integer & 0xFFFFFFFF) as u32;
        let payload_1 = ((integer >> 32) & 0xFFFFFFFF) as u32;
        let payload_2 = ((integer >> 64) & 0xFFFFFFFF) as u32;
        let payload_3 = ((integer >> 96) & 0xFFFFFFFF) as u32;

        self.add_payloads(
            [payload_0, payload_1, payload_2, payload_3],
            key,
            OperandType::U_128,
        )
    }

    pub fn add_i128(&mut self, integer: i128) -> ConstantId {
        let key = {
            let mut hasher = FxHasher::default();

            OperandType::I_128.hash(&mut hasher);
            integer.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(id) = self.interner.get(&key) {
            return *id;
        }

        let payload_0 = (integer & 0xFFFFFFFF) as u32;
        let payload_1 = ((integer >> 32) & 0xFFFFFFFF) as u32;
        let payload_2 = ((integer >> 64) & 0xFFFFFFFF) as u32;
        let payload_3 = ((integer >> 96) & 0xFFFFFFFF) as u32;

        self.add_payloads(
            [payload_0, payload_1, payload_2, payload_3],
            key,
            OperandType::I_128,
        )
    }

    pub fn add_f32(&mut self, float: f32) -> ConstantId {
        let bits = float.to_bits();
        let key = {
            let mut hasher = FxHasher::default();

            OperandType::F_32.hash(&mut hasher);
            bits.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(id) = self.interner.get(&key) {
            return *id;
        }

        self.add_payloads([bits], key, OperandType::F_32)
    }

    pub fn add_f64(&mut self, float: f64) -> ConstantId {
        let bits = float.to_bits();
        let key = {
            let mut hasher = FxHasher::default();

            OperandType::F_64.hash(&mut hasher);
            bits.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(id) = self.interner.get(&key) {
            return *id;
        }

        let low_payload = (bits & 0xFFFFFFFF) as u32;
        let high_payload = ((bits >> 32) & 0xFFFFFFFF) as u32;

        self.add_payloads([low_payload, high_payload], key, OperandType::F_64)
    }

    pub fn add_character(&mut self, character: char) -> ConstantId {
        let key = {
            let mut hasher = FxHasher::default();

            OperandType::CHARACTER.hash(&mut hasher);
            character.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(id) = self.interner.get(&key) {
            return *id;
        }

        let payload = character as u32;

        self.add_payloads([payload], key, OperandType::CHARACTER)
    }

    pub fn add_string(&mut self, str: &str) -> ConstantId {
        let key = {
            let mut hasher = FxHasher::default();

            OperandType::CHARACTER.hash(&mut hasher);
            str.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(id) = self.interner.get(&key) {
            return *id;
        }

        let start = self.string_pool.len() as u16;
        let length = str.len() as u16;
        let payload = ((start as u32) << 16) | (length as u32);

        self.string_pool.push_str(str);
        self.add_payloads([payload], key, OperandType::POINTER)
    }

    fn add_payloads<const COUNT: usize>(
        &mut self,
        payloads: [u32; COUNT],
        key: u64,
        tag: OperandType,
    ) -> ConstantId {
        let id = ConstantId(self.payloads.len() as u16);

        self.payloads.extend(payloads);
        self.tags.push(tag);
        self.interner.insert(key, id);

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

#[derive(Debug)]
pub enum ConstantListError {
    InvalidConstantPayload,
    MissingConstant(ConstantId),
}

impl<'a> AnnotatedError<'a> for ConstantListError {
    type Context = ();

    fn is_internal(&self) -> bool {
        true
    }

    fn add_report(&self, _: Self::Context, groups: &mut Vec<Group<'a>>) {
        self.add_internal_report(groups);
    }
}

#[cfg(test)]
mod tests {
    use crate::constant_list::*;

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
        let first_pool_length = table.string_pool.len();
        let second_id = table.add_string("foobar");
        let second_pool_length = table.string_pool.len();

        assert_eq!(first_id, second_id);
        assert_eq!(first_pool_length, second_pool_length);
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
