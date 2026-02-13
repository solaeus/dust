use std::hash::{Hash, Hasher};

use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHasher};

use crate::{
    compiler::error::{CompileError, InternalCompileError},
    source::Span,
};

#[derive(Debug)]
pub struct SymbolTable {
    pool: String,
    spans: IndexMap<u64, Span, FxBuildHasher>,
    next_anonymous_symbol_id: AnonymousSymbolId,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut symbol_table = Self {
            pool: String::new(),
            spans: IndexMap::default(),
            next_anonymous_symbol_id: AnonymousSymbolId(0),
        };

        let _dummy_symbol_id = symbol_table.add_anonymous_symbol();
        let _core_symbol_id = symbol_table.add_anonymous_symbol();

        assert_eq!(_dummy_symbol_id, SymbolId::DUMMY);
        assert_eq!(_core_symbol_id, SymbolId::CORE);

        symbol_table
    }

    pub fn add_named_symbol(&mut self, name: &str) -> SymbolId {
        let hash = {
            let mut hasher = FxHasher::default();

            name.hash(&mut hasher);

            hasher.finish()
        };

        if let Some(existing_index) = self.spans.get_index_of(&hash) {
            return SymbolId::Named(NamedSymbolId(existing_index as u32));
        }

        let index = self.spans.len() as u32;
        let span = Span::new(self.pool.len(), self.pool.len() + name.len());

        self.pool.push_str(name);
        self.spans.insert(hash, span);

        SymbolId::Named(NamedSymbolId(index))
    }

    pub fn add_anonymous_symbol(&mut self) -> SymbolId {
        let id = self.next_anonymous_symbol_id;
        self.next_anonymous_symbol_id.0 += 1;

        SymbolId::Anonymous(id)
    }

    pub fn get_symbol(&self, symbol: &SymbolId) -> Result<&str, CompileError> {
        symbol
            .as_span_index()
            .and_then(|index| self.spans.get_index(index))
            .and_then(|(_, span)| self.pool.get(span.as_usize_range()))
            .ok_or(CompileError::Internal(
                InternalCompileError::AnonymousSymbolLookup,
            ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum SymbolId {
    Anonymous(AnonymousSymbolId),
    Named(NamedSymbolId),
}

impl SymbolId {
    pub const DUMMY: SymbolId = Self::Anonymous(AnonymousSymbolId(0));
    pub const CORE: SymbolId = Self::Anonymous(AnonymousSymbolId(1));

    fn as_span_index(&self) -> Option<usize> {
        if let SymbolId::Named(id) = self {
            Some(id.0 as usize)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnonymousSymbolId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NamedSymbolId(u32);
