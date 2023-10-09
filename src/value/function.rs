use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Identifier, Item};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Function {
    parameters: Vec<Identifier>,
    body: Item,
}

impl Function {
    pub fn new(identifiers: Vec<Identifier>, items: Item) -> Self {
        Function {
            parameters: identifiers,
            body: items,
        }
    }

    pub fn identifiers(&self) -> &Vec<Identifier> {
        &self.parameters
    }

    pub fn body(&self) -> &Item {
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
