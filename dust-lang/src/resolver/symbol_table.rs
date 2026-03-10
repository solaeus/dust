use std::hash::{Hash, Hasher};

use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHasher};

use crate::source::Span;

#[derive(Debug)]
pub struct SymbolTable {
    pool: String,
    spans: IndexMap<u64, Span, FxBuildHasher>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            pool: String::new(),
            spans: IndexMap::default(),
        }
    }

    pub fn add_symbol(&mut self, name: &str) -> SymbolId {
        let hash = {
            let mut hasher = FxHasher::default();

            name.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(existing_index) = self.spans.get_index_of(&hash) {
            return SymbolId(existing_index as u32);
        }

        let id = SymbolId(self.spans.len() as u32);
        let span = Span::new(self.pool.len(), self.pool.len() + name.len());

        self.pool.push_str(name);
        self.spans.insert(hash, span);

        id
    }

    pub fn get_symbol(&self, id: &SymbolId) -> Result<&str, crate::resolver::error::ResolverError> {
        let (_, span) = self
            .spans
            .get_index(id.0 as usize)
            .ok_or(crate::resolver::error::ResolverError::MissingSymbol(*id))?;
        let symbol = &self.pool[span.as_usize_range()];

        Ok(symbol)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct SymbolId(u32);

impl SymbolId {
    pub const DUMMY: SymbolId = Self(u32::MAX);
}
