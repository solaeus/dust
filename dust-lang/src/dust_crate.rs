use std::sync::Arc;

use crate::{Chunk, ConstantTable};

pub enum DustCrate {
    Program(Program),
    Library(Module),
}

pub struct Program {
    pub name: Arc<String>,
    pub prototypes: Vec<Chunk>,
    pub constants: ConstantTable,
}

impl Program {
    pub fn new(name: Arc<String>, main_chunk: Chunk, constants: ConstantTable) -> Self {
        Self {
            name,
            prototypes: vec![main_chunk],
            constants,
        }
    }

    pub fn main_chunk(&self) -> &Chunk {
        self.prototypes
            .first()
            .expect("Program should always have a main chunk")
    }
}

pub struct Module;
