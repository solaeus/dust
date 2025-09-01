use serde::{Deserialize, Serialize};

use crate::{OperandType, Value};

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConstantTable {
    payloads: Vec<u64>,
    tags: Vec<OperandType>,
    string_pool: String,
}

impl ConstantTable {
    pub fn new() -> Self {
        Self {
            payloads: Vec::new(),
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

    pub fn add_character(&mut self, character: char) -> u16 {
        self.verify_table_length();

        let payload = character as u64;
        let index = self.payloads.len() as u16;

        self.payloads.push(payload);
        self.tags.push(OperandType::CHARACTER);

        index
    }

    pub fn add_float(&mut self, float: f64) -> u16 {
        self.verify_table_length();

        let payload = float.to_bits();
        let index = self.payloads.len() as u16;

        self.payloads.push(payload);
        self.tags.push(OperandType::FLOAT);

        index
    }

    pub fn add_integer(&mut self, integer: i64) -> u16 {
        self.verify_table_length();

        let payload = integer as u64;
        let index = self.payloads.len() as u16;

        self.payloads.push(payload);
        self.tags.push(OperandType::INTEGER);

        index
    }

    pub fn add_string(&mut self, string: &str) -> u16 {
        self.verify_table_length();
        self.verify_string_pool_length(string);

        let start = self.string_pool.len();

        self.string_pool.push_str(string);

        let end = self.string_pool.len();
        let payload = (start as u64) << 32 | (end as u64);
        let index = self.payloads.len() as u16;

        self.payloads.push(payload);
        self.tags.push(OperandType::STRING);

        index
    }

    pub fn concatenate_strings(&mut self, left_index: u16, right_index: u16) -> u16 {
        if left_index + 1 == right_index {
            let start = self.payloads[left_index as usize] >> 32;
            let end = self.payloads[right_index as usize] & 0xFFFFFFFF;
            let payload = (start << 32) | end;
            let index = self.payloads.len() as u16;

            self.payloads.push(payload);
            self.tags.push(OperandType::STRING);

            index
        } else {
            let left_payload = self.payloads[left_index as usize];
            let right_payload = self.payloads[right_index as usize];
            let left_start = (left_payload >> 32) as usize;
            let left_end = (left_payload & 0xFFFFFFFF) as usize;
            let right_start = (right_payload >> 32) as usize;
            let right_end = (right_payload & 0xFFFFFFFF) as usize;
            let left_string = &self.string_pool[left_start..left_end];
            let right_string = &self.string_pool[right_start..right_end];
            let concatenated_string = format!("{}{}", left_string, right_string);

            self.add_string(&concatenated_string)
        }
    }

    fn verify_string_pool_length(&self, new_string: &str) {
        let distance_to_max = u32::MAX as usize - self.string_pool.len();

        if new_string.len() > distance_to_max {
            panic!(
                "String pool overflow. Cannot store more than {} bytes in the string pool.",
                u32::MAX
            );
        }
    }

    fn verify_table_length(&self) {
        if self.payloads.len() > u16::MAX as usize {
            panic!(
                "Constant table overflow. Cannot store more than {} constants.",
                u16::MAX
            );
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
