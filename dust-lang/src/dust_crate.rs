use std::{collections::HashMap, sync::Arc};

use rustc_hash::FxBuildHasher;

use crate::{Chunk, ConstantTable, Resolver, resolver::DeclarationId, syntax_tree::SyntaxTree};

pub enum DustCrate {
    Program(Box<Program>),
    Library(Library),
}

pub struct Program {
    pub name: Arc<String>,
    pub prototypes: Vec<Chunk>,
    pub constants: ConstantTable,
    pub resolver: Resolver,
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
