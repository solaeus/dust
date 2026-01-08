use std::sync::Arc;

use crate::{constant_table::ConstantTable, prototype::Prototype};

pub enum DustCrate {
    Program(Arc<Program>),
}

pub struct Program {
    pub name_index: u16,
    pub constants: ConstantTable,
    pub prototypes: Vec<Prototype>,
}

impl Program {
    pub fn name(&self) -> &str {
        self.constants
            .get_string(self.name_index)
            .expect("Program name index does not point to a string constant")
    }

    pub fn main_prototype(&self) -> &Prototype {
        self.prototypes
            .first()
            .expect("Program does not have a main prototype")
    }
}
