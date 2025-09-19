use std::{collections::HashMap, sync::Arc};

use rustc_hash::FxBuildHasher;

use crate::{Chunk, ConstantTable, resolver::DeclarationId, syntax_tree::SyntaxTree};

pub enum DustCrate {
    Program(Arc<Program>),
    Library(Library),
}

pub struct Program {
    pub name: String,
    pub prototypes: Vec<Chunk>,
    pub constants: ConstantTable,
}

impl Program {
    pub fn main_chunk(&self) -> &Chunk {
        self.prototypes
            .first()
            .expect("Program should always have a main chunk")
    }
}

pub struct Library {
    pub name: Arc<String>,
    pub file_trees: HashMap<DeclarationId, SyntaxTree, FxBuildHasher>,
}
