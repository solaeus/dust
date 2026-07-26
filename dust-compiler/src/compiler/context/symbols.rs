use std::{
    fmt::Write,
    hash::{Hash, Hasher},
};

use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHasher};
use serde::{Deserialize, Serialize};

use crate::{
    compiler::error::CompileError,
    source::{Position, Span},
};

#[derive(Debug)]
pub struct Symbols {
    pool: String,
    spans: IndexMap<u64, Span, FxBuildHasher>,
}

impl Symbols {
    pub fn new() -> Self {
        Self {
            pool: String::new(),
            spans: IndexMap::default(),
        }
    }

    pub fn symbol_count(&self) -> usize {
        self.spans.len()
    }

    pub fn add_symbol(&mut self, name: &str) -> SymbolId {
        let hash = {
            let mut hasher = FxHasher::default();

            hasher.write_u8(0);
            name.hash(&mut hasher);
            hasher.finish()
        };

        if let Some(existing_index) = self.spans.get_index_of(&hash) {
            return SymbolId(existing_index as u32);
        }

        let id = SymbolId(self.spans.len() as u32);
        let start = self.pool.len() as u32;

        self.pool.push_str(name);

        let end = self.pool.len() as u32;

        self.spans.insert(hash, Span::new(start, end));

        id
    }

    pub fn add_index_symbol(&mut self, index: u32) -> SymbolId {
        let hash = {
            let mut hasher = FxHasher::default();

            hasher.write_u8(1);
            index.hash(&mut hasher);
            hasher.finish()
        };

        if let Some(existing_index) = self.spans.get_index_of(&hash) {
            return SymbolId(existing_index as u32);
        }

        let id = SymbolId(self.spans.len() as u32);
        let start = self.pool.len() as u32;

        let _ = write!(&mut self.pool, "{index}");

        self.spans
            .insert(hash, Span::new(start, self.pool.len() as u32));

        id
    }

    pub fn add_impl_symbol(&mut self, position: Position) -> SymbolId {
        let id = SymbolId(self.spans.len() as u32);
        let start = self.pool.len();

        let _ = write!(
            &mut self.pool,
            "impl@{}:{}",
            position.code_id.inner(),
            position.span
        );

        let hash = {
            let mut hasher = FxHasher::default();

            hasher.write_u8(2);
            self.pool[start..].hash(&mut hasher);
            hasher.finish()
        };

        self.spans.insert(hash, Span::new(start, self.pool.len()));

        id
    }

    pub fn get_symbol(&self, id: &SymbolId) -> Result<&str, CompileError> {
        let span = self.spans[id.0 as usize];
        let symbol = &self.pool[span.as_usize_range()];

        Ok(symbol)
    }
}

impl Default for Symbols {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SymbolId(#[cfg(test)] pub(crate) u32, #[cfg(not(test))] u32);

impl SymbolId {
    pub const PLACEHOLDER: Self = Self(u32::MAX);
}
