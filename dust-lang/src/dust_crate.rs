use std::sync::Arc;

use crate::{constant_table::ConstantTable, prototype::Prototype, source::Source};

pub enum DustCrate {
    Program(Arc<Program>),
}

pub struct Program {
    pub name: String,
    pub source: Source,
    pub constants: ConstantTable,
    pub prototypes: Vec<Prototype>,
}

impl Program {
    pub fn main_prototype(&self) -> &Prototype {
        self.prototypes
            .first()
            .expect("Program should always have a main prototype")
    }
}
