use std::sync::Arc;

use crate::{Chunk, ConstantTable, Resolver};

pub enum DustCrate {
    Program(Program),
    Library(Module),
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

pub struct Module;
