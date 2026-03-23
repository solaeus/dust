use std::{
    fmt::Write,
    hash::{Hash, Hasher},
};

use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHasher};
use serde::{Deserialize, Serialize};

use crate::{resolver::error::ResolverError, source::Span};

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
        self.spans
            .insert(hash, Span::new(start, self.pool.len() as u32));

        id
    }

    pub fn add_index_symbol(&mut self, index: usize) -> SymbolId {
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

    pub fn get_symbol(&self, id: &SymbolId) -> Result<&str, ResolverError> {
        let (_, span) = self
            .spans
            .get_index(id.0 as usize)
            .ok_or(crate::resolver::error::ResolverError::MissingSymbol(*id))?;
        let symbol = &self.pool[span.as_usize_range()];

        Ok(symbol)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SymbolId(u32);
