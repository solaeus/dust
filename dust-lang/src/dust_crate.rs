use std::sync::Arc;

use crate::{constant_table::ConstantTable, prototype::PrototypeList, syntax::SyntaxTree};

pub enum DustCrate {
    Program(Arc<Program>),
    Library(Vec<SyntaxTree>),
}

pub struct Program {
    name: String,
    pub prototypes: PrototypeList,
    pub(crate) constants: ConstantTable,
}

impl Program {
    const DEFAULT_NAME: &str = "dust_program";

    pub fn new(name: Option<String>, constants: ConstantTable, prototypes: PrototypeList) -> Self {
        let name = name.unwrap_or_else(|| Self::DEFAULT_NAME.to_string());

        Self {
            name,
            prototypes,
            constants,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}
