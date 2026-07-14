use serde::{Deserialize, Serialize};

use crate::{constants::Constants, dust_type::DustType, prototype::Prototype};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Program {
    name: String,
    return_type: DustType,
    prototypes: Vec<Prototype>,
    constants: Constants,
}

impl Program {
    pub fn new(
        name: String,
        return_type: DustType,
        constants: Constants,
        prototypes: Vec<Prototype>,
    ) -> Self {
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

    pub fn prototypes(&self) -> &Vec<Prototype> {
        &self.prototypes
    }

    pub fn constants(&self) -> &Constants {
        &self.constants
    }
}
