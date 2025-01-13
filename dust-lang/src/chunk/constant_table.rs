use serde::{Deserialize, Serialize};

use crate::DustString;

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ConstantTable {
    pub r#true: bool,
    pub r#false: bool,
    pub bytes: Vec<u8>,
    pub characters: Vec<char>,
    pub floats: Vec<f64>,
    pub integers: Vec<i64>,
    pub strings: Vec<DustString>,
}

impl ConstantTable {
    pub fn new() -> Self {
        Self {
            r#true: false,
            r#false: false,
            bytes: Vec::new(),
            characters: Vec::new(),
            floats: Vec::new(),
            integers: Vec::new(),
            strings: Vec::new(),
        }
    }

    #[cfg(debug_assertions)]
    pub fn with_data(
        booleans: (bool, bool),
        bytes: impl Into<Vec<u8>>,
        characters: impl Into<Vec<char>>,
        floats: impl Into<Vec<f64>>,
        integers: impl Into<Vec<i64>>,
        strings: impl Into<Vec<DustString>>,
    ) -> Self {
        Self {
            r#true: booleans.0,
            r#false: booleans.1,
            bytes: bytes.into(),
            characters: characters.into(),
            floats: floats.into(),
            integers: integers.into(),
            strings: strings.into(),
        }
    }

    pub fn len(&self) -> usize {
        (if self.r#true { 1 } else { 0 })
            + (if self.r#false { 1 } else { 0 })
            + self.bytes.len()
            + self.characters.len()
            + self.floats.len()
            + self.integers.len()
            + self.strings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
