use std::sync::Arc;

use crate::{chunk::Chunk, constant_table::ConstantTable, source::Source};

pub enum DustCrate {
    Program(Arc<Program>),
}

pub struct Program {
    pub source: Source,
    pub constants: ConstantTable,
    pub prototypes: Vec<Chunk>,
}

impl Program {
    pub fn main_chunk(&self) -> &Chunk {
        self.prototypes
            .first()
            .expect("Program should always have a main chunk")
    }
}
