use std::sync::Arc;

use crate::{
    constant_table::{ConstantId, ConstantTable},
    prototype::Prototype,
};

pub enum DustCrate {
    Program(Arc<Program>),
}

pub struct Program {
    pub name_id: ConstantId,
    pub constants: ConstantTable,
    pub prototypes: Vec<Prototype>,
}

impl Program {
    pub fn name(&self) -> &str {
        self.constants
            .get_string(self.name_id)
            .expect("Invalid program: name ID does not correspond to a string constant")
    }

    pub fn main_prototype(&self) -> &Prototype {
        self.prototypes
            .first()
            .expect("Invalid program: no prototypes found")
    }
}
