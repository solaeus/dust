use serde::{Deserialize, Serialize};

use crate::DustString;

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ConstantTable {
    pub bytes: Vec<u8>,
    pub characters: Vec<char>,
    pub floats: Vec<f64>,
    pub integers: Vec<i64>,
    pub strings: Vec<DustString>,
}

impl ConstantTable {
    pub fn new() -> Self {
        Self {
            bytes: Vec::with_capacity(0),
            characters: Vec::with_capacity(0),
            floats: Vec::with_capacity(0),
            integers: Vec::with_capacity(0),
            strings: Vec::with_capacity(0),
        }
    }

    #[cfg(debug_assertions)]
    pub fn with_data(
        bytes: impl Into<Vec<u8>>,
        characters: impl Into<Vec<char>>,
        floats: impl Into<Vec<f64>>,
        integers: impl Into<Vec<i64>>,
        strings: impl Into<Vec<DustString>>,
    ) -> Self {
        Self {
            bytes: bytes.into(),
            characters: characters.into(),
            floats: floats.into(),
            integers: integers.into(),
            strings: strings.into(),
        }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
            + self.characters.len()
            + self.floats.len()
            + self.integers.len()
            + self.strings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
            && self.characters.is_empty()
            && self.floats.is_empty()
            && self.integers.is_empty()
            && self.strings.is_empty()
    }

    pub fn insert_byte(&mut self, byte: u8) -> u16 {
        if let Some(index) = self.bytes.iter().position(|&probe| probe == byte) {
            index as u16
        } else {
            let index = self.bytes.len() as u16;

            self.bytes.push(byte);

            index
        }
    }

    pub fn insert_character(&mut self, character: char) -> u16 {
        if let Some(index) = self.characters.iter().position(|&probe| probe == character) {
            index as u16
        } else {
            let index = self.characters.len() as u16;

            self.characters.push(character);

            index
        }
    }

    pub fn insert_float(&mut self, float: f64) -> u16 {
        if let Some(index) = self.floats.iter().position(|&probe| probe == float) {
            index as u16
        } else {
            let index = self.floats.len() as u16;

            self.floats.push(float);

            index
        }
    }

    pub fn insert_integer(&mut self, integer: i64) -> u16 {
        if let Some(index) = self.integers.iter().position(|&probe| probe == integer) {
            index as u16
        } else {
            let index = self.integers.len() as u16;

            self.integers.push(integer);

            index
        }
    }

    pub fn insert_string(&mut self, string: DustString) -> u16 {
        if let Some(index) = self.strings.iter().position(|probe| probe == &string) {
            index as u16
        } else {
            let index = self.strings.len() as u16;

            self.strings.push(string);

            index
        }
    }

    pub fn get_byte(&self, index: u16) -> Option<u8> {
        self.bytes.get(index as usize).copied()
    }

    pub fn get_character(&self, index: u16) -> Option<char> {
        self.characters.get(index as usize).copied()
    }

    pub fn get_float(&self, index: u16) -> Option<f64> {
        self.floats.get(index as usize).copied()
    }

    pub fn get_integer(&self, index: u16) -> Option<i64> {
        self.integers.get(index as usize).copied()
    }

    pub fn get_string(&self, index: u16) -> Option<&DustString> {
        self.strings.get(index as usize)
    }
}
