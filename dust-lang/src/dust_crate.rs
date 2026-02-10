use std::sync::Arc;

use crate::{
    constant_table::{ConstantId, ConstantTable},
    prototype::Prototype,
    parser::syntax::SyntaxTree,
};

pub enum DustCrate {
    Program(Arc<Program>),
    Library(Vec<SyntaxTree>),
}

pub struct Program {
    name_id: ConstantId,
    pub(crate) constants: ConstantTable,
    pub(crate) prototypes: Vec<Prototype>,
}

impl Program {
    pub const DEFAULT_NAME: &str = "dust_program";

    pub fn new(
        name: Option<&str>,
        mut constants: ConstantTable,
        prototypes: Vec<Prototype>,
    ) -> Self {
        debug_assert!(!prototypes.is_empty());

        let name_id = constants.add_string(name.unwrap_or(Self::DEFAULT_NAME));

        Self {
            name_id,
            constants,
            prototypes,
        }
    }

    pub fn name(&self) -> &str {
        self.constants.get_string(self.name_id)
    }

    pub fn main_prototype(&self) -> &Prototype {
        self.prototypes
            .first()
            .expect("Invalid program: no prototypes found")
    }
}
