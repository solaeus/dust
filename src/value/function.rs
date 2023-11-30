use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Block, Identifier, TypeDefinition};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Function {
    parameters: Vec<(Identifier, TypeDefinition)>,
    body: Block,
    return_type: TypeDefinition,
}

impl Function {
    pub fn new(
        parameters: Vec<(Identifier, TypeDefinition)>,
        body: Block,
        return_type: TypeDefinition,
    ) -> Self {
        Self {
            parameters,
            body,
            return_type,
        }
    }

    pub fn parameters(&self) -> &Vec<(Identifier, TypeDefinition)> {
        &self.parameters
    }

    pub fn body(&self) -> &Block {
        &self.body
    }

    pub fn return_type(&self) -> &TypeDefinition {
        &self.return_type
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Function {{ parameters: {:?}, body: {:?} }}",
            self.parameters, self.body
        )
    }
}
