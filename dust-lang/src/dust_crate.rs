use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{Chunk, Module, Program};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DustCrate {
    Library(Module),
    Program(Box<Program>),
}

impl DustCrate {
    pub fn library(module: Module) -> Self {
        Self::Library(module)
    }

    pub fn program(main_chunk: Chunk, cell_count: u16, prototypes: Arc<Vec<Arc<Chunk>>>) -> Self {
        Self::Program(Box::new(Program {
            main_chunk,
            cell_count,
            prototypes,
        }))
    }
}
