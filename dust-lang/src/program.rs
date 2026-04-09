use serde::{Deserialize, Serialize};

use crate::{constants::Constants, dust_type::DustType, prototype::PrototypeList};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Program {
    name: String,
    return_type: DustType,

    pub prototypes: PrototypeList,
    pub constants: Constants,
}

impl Program {
    pub const DEFAULT_NAME: &str = "dust_program";

    pub fn new(
        name: Option<String>,
        return_type: DustType,
        constants: Constants,
        prototypes: PrototypeList,
    ) -> Self {
        let name = name.unwrap_or_else(|| Self::DEFAULT_NAME.to_string());

        Self {
            name,
            return_type,
            prototypes,
            constants,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn return_type(&self) -> &DustType {
        &self.return_type
    }
}
