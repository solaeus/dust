use serde::{Deserialize, Serialize};

use crate::{Chunk, Module};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DustCrate {
    Library(Module),
    Program(Box<Program>),
}

impl DustCrate {
    pub fn library(module: Module) -> Self {
        Self::Library(module)
    }

    pub fn program(main_chunk: Chunk, cell_count: u16) -> Self {
        Self::Program(Box::new(Program {
            main_chunk,
            cell_count,
        }))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Program {
    pub main_chunk: Chunk,
    pub cell_count: u16,
}
