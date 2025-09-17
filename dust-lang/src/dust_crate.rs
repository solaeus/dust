use std::sync::Arc;

use crate::Chunk;

pub enum DustCrate {
    Program(Program),
    Library(Module),
}

pub struct Program {
    pub name: Arc<String>,
    pub prototypes: Vec<Chunk>,
    pub cell_count: usize,
}

impl Program {
    pub fn new(name: Arc<String>, main_chunk: Chunk) -> Self {
        Self {
            name,
            prototypes: vec![main_chunk],
            cell_count: 0,
        }
    }

    pub fn main_chunk(&self) -> &Chunk {
        self.prototypes
            .first()
            .expect("Program should always have a main chunk")
    }
}

pub struct Module;
