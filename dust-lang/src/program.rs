use serde::{Deserialize, Serialize};

use crate::{constant_table::ConstantTable, prototype::PrototypeList};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Program {
    name: String,
    pub prototypes: PrototypeList,
    pub(crate) constants: ConstantTable,
}

impl Program {
    pub const DEFAULT_NAME: &str = "dust_program";

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
