use std::{collections::HashMap, sync::Arc};

use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

use crate::{Chunk, ConstantTable, resolver::DeclarationId, syntax_tree::SyntaxTree};

pub enum DustCrate {
    Program(Arc<Program>),
    Library(Library),
}

pub struct Program {
    pub name: String,
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

pub struct Library {
    pub name: Arc<String>,
    pub file_trees: HashMap<DeclarationId, SyntaxTree, FxBuildHasher>,
}
