use serde::{Deserialize, Serialize};

use crate::OperandType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConstantId(pub u16);

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConstantTable {
    payloads: Vec<u64>,
    tags: Vec<OperandType>,
    string_pool: String,
}

impl ConstantTable {
    pub fn new(constant_count: usize, total_string_length: usize) -> Self {
        Self {
            payloads: Vec::with_capacity(constant_count),
            tags: Vec::with_capacity(constant_count),
            string_pool: String::with_capacity(total_string_length),
        }
    }

    pub fn add_integer(&mut self, integer: i64) -> ConstantId {
        self.verify_table_length();

        let payload = integer as u64;
        let index = self.payloads.len() as u16;

        self.payloads.push(payload);
        self.tags.push(OperandType::INTEGER);

        ConstantId(index)
    }

    pub fn add_string(&mut self, string: &str) -> ConstantId {
        self.verify_table_length();
        self.verify_string_pool_length(string);

        let start = self.string_pool.len();

        self.string_pool.push_str(string);

        let end = self.string_pool.len();
        let payload = (start as u64) << 32 | (end as u64);
        let index = self.payloads.len() as u16;

        self.payloads.push(payload);
        self.tags.push(OperandType::STRING);

        ConstantId(index)
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
