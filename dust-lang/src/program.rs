use serde::{Deserialize, Serialize};

use crate::{constant_list::ConstantList, prototype::PrototypeList};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Program {
    name: String,
    pub prototypes: PrototypeList,
    pub(crate) constants: ConstantList,
}

impl Program {
    pub const DEFAULT_NAME: &str = "dust_program";

    pub fn new(name: Option<String>, constants: ConstantList, prototypes: PrototypeList) -> Self {
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
