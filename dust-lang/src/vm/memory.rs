use std::sync::Arc;

use crate::{DustString, List, StrippedChunk};

#[derive(Clone, Debug)]
pub enum Object {
    Empty,
    Function(Arc<StrippedChunk>),
    ValueList(List<StrippedChunk>),
    String(DustString),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Register(u64);

impl Register {
    pub fn boolean(boolean: bool) -> Self {
        Register(boolean as u64)
    }

    pub fn as_boolean(&self) -> bool {
        self.0 != 0
    }

    pub fn byte(byte: u8) -> Self {
        Register(byte as u64)
    }

    pub fn as_byte(&self) -> u8 {
        self.0 as u8
    }

    pub fn character(character: char) -> Self {
        Register(character as u64)
    }

    pub fn as_character(&self) -> char {
        char::from_u32(self.0 as u32).unwrap_or_default()
    }

    pub fn float(float: f64) -> Self {
        Register(float.to_bits())
    }

    pub fn as_float(&self) -> f64 {
        f64::from_bits(self.0)
    }

    pub fn integer(integer: i64) -> Self {
        let bytes = integer.to_le_bytes();

        Register(u64::from_le_bytes(bytes))
    }

    pub fn as_integer(&self) -> i64 {
        let bytes = self.0.to_le_bytes();

        i64::from_le_bytes(bytes)
    }

    pub fn index(index: usize) -> Self {
        Register(index as u64)
    }

    pub fn as_index(&self) -> usize {
        self.0 as usize
    }
}
