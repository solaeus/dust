use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Identifier, Statement};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Function {
    parameters: Vec<Identifier>,
    body: Box<Statement>,
}

impl Function {
    pub fn new(identifiers: Vec<Identifier>, items: Statement) -> Self {
        Function {
            parameters: identifiers,
            body: Box::new(items),
        }
    }

    pub fn identifiers(&self) -> &Vec<Identifier> {
        &self.parameters
    }

    pub fn body(&self) -> &Statement {
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
