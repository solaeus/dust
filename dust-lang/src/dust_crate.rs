use serde::{Deserialize, Serialize};

use crate::Module;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DustCrate<C> {
    Library(Module<C>),
    Program(Box<Program<C>>),
}

impl<C> DustCrate<C> {
    pub fn library(module: Module<C>) -> Self {
        Self::Library(module)
    }

    pub fn program(main_chunk: C, cell_count: u16) -> Self {
        Self::Program(Box::new(Program {
            main_chunk,
            cell_count,
        }))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Program<C> {
    pub main_chunk: C,
    pub cell_count: u16,
}
