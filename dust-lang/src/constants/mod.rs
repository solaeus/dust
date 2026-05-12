#[cfg(test)]
mod tests;

pub mod value;

use std::{
    char,
    collections::HashMap,
    hash::{Hash, Hasher},
};

use annotate_snippets::Group;
use rustc_hash::{FxBuildHasher, FxHasher};
use serde::{Deserialize, Serialize};

use crate::{error::DustError, instruction::OperandType};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Constants {
    payloads: Vec<u32>,
    string_pool: String,
}

impl Constants {
    pub fn get_u32(&self, index: u16) -> Result<u32, ConstantsError> {
        let payload = *self
            .payloads
            .get(index as usize)
            .ok_or(ConstantsError::MissingConstant(index))?;

        Ok(payload)
    }

    pub fn get_i32(&self, index: u16) -> Result<i32, ConstantsError> {
        let payload = *self
            .payloads
            .get(index as usize)
            .ok_or(ConstantsError::MissingConstant(index))?;

        Ok(payload as i32)
    }

    pub fn get_u64(&self, index: u16) -> Result<u64, ConstantsError> {
        let index = index as usize;

        if index + 1 >= self.payloads.len() {
            return Err(ConstantsError::MissingConstant(index as u16));
        }

        let low = self.payloads[index] as u64;
        let high = self.payloads[index + 1] as u64;
        let decoded = (high << 32) | low;

        Ok(decoded)
    }

    pub fn get_i64(&self, index: u16) -> Result<i64, ConstantsError> {
        let payload_range = index as usize..(index + 2) as usize;
        let payloads = self
            .payloads
            .get(payload_range)
            .ok_or(ConstantsError::MissingConstant(index))?;
        let decoded = (payloads[1] as i64) << 32 | (payloads[0] as i64);

        Ok(decoded)
    }

    pub fn get_u128(&self, index: u16) -> Result<u128, ConstantsError> {
        let payload_range = index as usize..(index + 4) as usize;
        let payloads = self
            .payloads
            .get(payload_range)
            .ok_or(ConstantsError::MissingConstant(index))?;
        let decoded = (payloads[3] as u128) << 96
            | (payloads[2] as u128) << 64
            | (payloads[1] as u128) << 32
            | (payloads[0] as u128);

        Ok(decoded)
    }

    pub fn get_i128(&self, index: u16) -> Result<i128, ConstantsError> {
        let payload_range = index as usize..(index + 4) as usize;
        let payloads = self
            .payloads
            .get(payload_range)
            .ok_or(ConstantsError::MissingConstant(index))?;
        let decoded = (payloads[3] as i128) << 96
            | (payloads[2] as i128) << 64
            | (payloads[1] as i128) << 32
            | (payloads[0] as i128);

        Ok(decoded)
    }

    pub fn get_f32(&self, index: u16) -> Result<f32, ConstantsError> {
        let payload = *self
            .payloads
            .get(index as usize)
            .ok_or(ConstantsError::MissingConstant(index))?;

        Ok(f32::from_bits(payload))
    }

    pub fn get_f64(&self, index: u16) -> Result<f64, ConstantsError> {
        let payload_range = index as usize..(index + 2) as usize;
        let payloads = self
            .payloads
            .get(payload_range)
            .ok_or(ConstantsError::MissingConstant(index))?;
        let payload = (payloads[1] as u64) << 32 | (payloads[0] as u64);

        Ok(f64::from_bits(payload))
    }

    pub fn get_character(&self, index: u16) -> Result<char, ConstantsError> {
        let payload = *self
            .payloads
            .get(index as usize)
            .ok_or(ConstantsError::MissingConstant(index))?;

        char::from_u32(payload).ok_or(ConstantsError::InvalidConstantPayload)
    }

    pub fn get_string(&self, index: u16) -> Result<&str, ConstantsError> {
        let payload = *self
            .payloads
            .get(index as usize)
            .ok_or(ConstantsError::MissingConstant(index))?;
        let start = (payload >> 16) as usize;
        let end = (payload & 0xFFFF) as usize;

        self.string_pool
            .get(start..end)
            .ok_or(ConstantsError::InvalidConstantPayload)
    }

    pub fn get_string_raw_parts(&self, index: u16) -> Result<(*const u8, usize), ConstantsError> {
        self.get_string(index).map(|str| (str.as_ptr(), str.len()))
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConstantsBuilder {
    payloads: Vec<u32>,
    tags: Vec<OperandType>,
    string_pool: String,
    interner: HashMap<u64, ConstantId, FxBuildHasher>,
}

impl ConstantsBuilder {
    pub fn new() -> Self {
        Self {
            payloads: Vec::new(),
            tags: Vec::new(),
            interner: HashMap::default(),
            string_pool: String::new(),
        }
    }

    pub fn build(self) -> (Constants, Vec<OperandType>) {
        (
            Constants {
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

        self.add_constant([integer], key, OperandType::U_32)
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

        self.add_constant([integer as u32], key, OperandType::I_32)
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

        self.add_constant([low_payload, high_payload], key, OperandType::U_64)
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

        self.add_constant([low_payload, high_payload], key, OperandType::I_64)
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

        self.add_constant(
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

        self.add_constant(
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

        self.add_constant([bits], key, OperandType::F_32)
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

        self.add_constant([low_payload, high_payload], key, OperandType::F_64)
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

        self.add_constant([payload], key, OperandType::CHARACTER)
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

        self.string_pool.push_str(str);

        let end = self.string_pool.len() as u16;
        let payload = ((start as u32) << 16) | (end as u32);

        self.add_constant([payload], key, OperandType::POINTER)
    }

    fn add_constant<const COUNT: usize>(
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

#[derive(Clone, Debug)]
pub enum ConstantsError {
    InvalidConstantPayload,
    MissingConstant(u16),
}

impl<'a> DustError<'a> for ConstantsError {
    type Info = ();

    fn add_report(&self, _: Self::Info, groups: &mut Vec<Group<'a>>) {
        self.add_internal_report(groups);
    }
}
