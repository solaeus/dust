use std::{
    hash::{Hash, Hasher},
    ops::Range,
};

use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHasher};
use serde::{Deserialize, Serialize};

use crate::{OperandType, Value};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConstantTable {
    payloads: IndexMap<u64, u64, FxBuildHasher>,
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

    pub fn iter<'a>(&'a self) -> ConstantTableIterator<'a> {
        ConstantTableIterator {
            table: self,
            index: 0,
        }
    }

    pub fn get_string_pool(&self, range: Range<usize>) -> &str {
        &self.string_pool[range]
    }

    pub fn add_character(&mut self, character: char) -> u16 {
        let payload = character as u64;
        let index = self.payloads.len() as u16;
        let hash = {
            let mut hasher = FxHasher::default();

            character.hash(&mut hasher);

            hasher.finish()
        };

        self.payloads.insert(hash, payload);
        self.tags.push(OperandType::CHARACTER);

        index
    }

    pub fn add_float(&mut self, float: f64) -> u16 {
        let payload = float.to_bits();
        let index = self.payloads.len() as u16;
        let hash = {
            let mut hasher = FxHasher::default();

            payload.hash(&mut hasher);

            hasher.finish()
        };

        self.payloads.insert(hash, payload);
        self.tags.push(OperandType::FLOAT);

        index
    }

    pub fn add_integer(&mut self, integer: i64) -> u16 {
        let payload = integer as u64;
        let index = self.payloads.len() as u16;
        let hash = {
            let mut hasher = FxHasher::default();

            integer.hash(&mut hasher);

            hasher.finish()
        };

        self.payloads.insert(hash, payload);
        self.tags.push(OperandType::INTEGER);

        index
    }

    pub fn add_string(&mut self, string: &str) -> (u32, u32) {
        let hash = {
            let mut hasher = FxHasher::default();

            string.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(existing_index) = self.payloads.get_index_of(&hash) {
            let payload = self.payloads[existing_index];
            let start = (payload >> 32) as u32;
            let end = (payload & 0xFFFFFFFF) as u32;

            (start, end)
        } else {
            let start = self.string_pool.len();

            self.string_pool.push_str(string);

            let end = self.string_pool.len();
            let payload = (start as u64) << 32 | (end as u64);

            self.payloads.insert(hash, payload);
            self.tags.push(OperandType::STRING);

            (start as u32, end as u32)
        }
    }

    pub fn push_str_to_string_pool(&mut self, string: &str) -> (u32, u32) {
        let hash = {
            let mut hasher = FxHasher::default();

            string.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(existing_index) = self.payloads.get_index_of(&hash) {
            let payload = self.payloads[existing_index];
            let start = (payload >> 32) as u32;
            let end = (payload & 0xFFFFFFFF) as u32;

            (start, end)
        } else {
            let start = self.string_pool.len() as u32;

            self.string_pool.push_str(string);

            let end = self.string_pool.len() as u32;

            (start, end)
        }
    }

    pub fn add_pooled_string(&mut self, start: u32, end: u32) -> u16 {
        let string = &self.string_pool[start as usize..end as usize];
        let hash = {
            let mut hasher = FxHasher::default();

            string.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(existing_index) = self.payloads.get_index_of(&hash) {
            existing_index as u16
        } else {
            let payload = (start as u64) << 32 | (end as u64);
            let index = self.payloads.len() as u16;

            self.payloads.insert(hash, payload);
            self.tags.push(OperandType::STRING);

            index
        }
    }
}

pub struct ConstantTableIterator<'a> {
    table: &'a ConstantTable,
    index: usize,
}

impl Iterator for ConstantTableIterator<'_> {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.table.payloads.len() {
            return None;
        }

        let tag = self.table.tags[self.index];
        let payload = self.table.payloads[self.index];
        let value = match tag {
            OperandType::CHARACTER => Value::Character(std::char::from_u32(payload as u32)?),
            OperandType::INTEGER => Value::Integer(payload as i64),
            OperandType::STRING => {
                let start = (payload >> 32) as usize;
                let end = (payload & 0xFFFFFFFF) as usize;

                let string = self.table.string_pool.get(start..end)?;

                Value::String(string.to_string())
            }
            _ => todo!(),
        };

        self.index += 1;

        Some(value)
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
