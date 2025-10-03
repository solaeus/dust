use std::sync::Arc;

use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

use crate::{chunk::Chunk, constant_table::ConstantTable, resolver::DeclarationId, source::Source};

pub enum DustCrate {
    Program(Arc<Program>),
}

pub struct Program {
    pub source: Source,
    pub constants: ConstantTable,
    pub prototypes: IndexMap<DeclarationId, Chunk, FxBuildHasher>,
}

impl Program {
    pub fn main_chunk(&self) -> &Chunk {
        self.prototypes
            .values()
            .next()
            .expect("Program should always have a main chunk")
    }
}
