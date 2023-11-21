use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Block, Identifier, Type};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Function {
    parameters: Option<Vec<(Identifier, Type)>>,
    body: Box<Block>,
}

impl Function {
    pub fn new(parameters: Option<Vec<(Identifier, Type)>>, body: Block) -> Self {
        Function {
            parameters,
            body: Box::new(body),
        }
    }

    pub fn identifiers(&self) -> &Option<Vec<(Identifier, Type)>> {
        &self.parameters
    }

    pub fn body(&self) -> &Block {
        &self.body
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "function < {:?} > {{ {:?} }}", // TODO: Correct this output
            self.parameters, self.body
        )
    }
}
