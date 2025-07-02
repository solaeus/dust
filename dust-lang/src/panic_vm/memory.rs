use std::sync::{Arc, RwLock};

use crate::{Chunk, DustString, List};

#[derive(Debug)]
pub struct Memory<C> {
    pub registers: Vec<Register>,
    pub objects: Vec<Object<C>>,
}

impl<C: Chunk> Memory<C> {
    pub fn new(chunk: &C) -> Self {
        Memory {
            registers: Vec::with_capacity(chunk.register_count()),
            objects: Vec::with_capacity(0),
        }
    }

    pub fn create_registers(&mut self, count: usize) {
        self.registers.reserve_exact(count);
        self.registers.resize(count, Register(0));
    }
}

#[derive(Debug)]
pub enum Object<C> {
    List(List<C>),
    Function(Arc<C>),
}

#[derive(Clone, Copy, Debug)]
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
