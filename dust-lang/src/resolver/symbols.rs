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

    next_impl_index: u32,
    next_slice_index: u32,
    next_self_index: u32,
}

impl Symbols {
    pub fn new() -> Self {
        Self {
            pool: String::new(),
            spans: IndexMap::default(),
            next_slice_index: 0,
            next_self_index: 0,
            next_impl_index: 0,
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

    pub fn add_impl_symbol(&mut self) -> SymbolId {
        let hash = {
            let mut hasher = FxHasher::default();

            hasher.write_u8(2);
            self.next_impl_index.hash(&mut hasher);
            hasher.finish()
        };

        if let Some(existing_index) = self.spans.get_index_of(&hash) {
            return SymbolId(existing_index as u32);
        }

        let id = SymbolId(self.spans.len() as u32);
        let start = self.pool.len() as u32;

        let _ = write!(&mut self.pool, "impl#{}", self.next_impl_index);
        self.next_impl_index += 1;

        self.spans
            .insert(hash, Span::new(start, self.pool.len() as u32));

        id
    }

    pub fn add_slice_symbol(&mut self) -> SymbolId {
        let hash = {
            let mut hasher = FxHasher::default();

            hasher.write_u8(3);
            self.next_slice_index.hash(&mut hasher);
            hasher.finish()
        };

        if let Some(existing_index) = self.spans.get_index_of(&hash) {
            return SymbolId(existing_index as u32);
        }

        let id = SymbolId(self.spans.len() as u32);
        let start = self.pool.len() as u32;

        let _ = write!(&mut self.pool, "[#{}]", self.next_slice_index);
        self.next_slice_index += 1;

        self.spans
            .insert(hash, Span::new(start, self.pool.len() as u32));

        id
    }

    pub fn add_self_symbol(&mut self) -> SymbolId {
        let hash = {
            let mut hasher = FxHasher::default();

            hasher.write_u8(4);
            self.next_self_index.hash(&mut hasher);
            hasher.finish()
        };

        if let Some(existing_index) = self.spans.get_index_of(&hash) {
            return SymbolId(existing_index as u32);
        }

        let id = SymbolId(self.spans.len() as u32);
        let start = self.pool.len() as u32;

        let _ = write!(&mut self.pool, "Self#{}", self.next_self_index);
        self.next_self_index += 1;

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
pub struct SymbolId(#[cfg(test)] pub(crate) u32, #[cfg(not(test))] u32);

impl SymbolId {
    pub const PLACEHOLDER: Self = Self(u32::MAX);
}
