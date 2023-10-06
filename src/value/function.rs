use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{EvaluatorTree, Identifier, Result, Statement, Value, VariableMap};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Function {
    identifiers: Vec<Identifier>,
    statements: Vec<Statement>,
}

impl Function {
    pub fn new(identifiers: Vec<Identifier>, statements: Vec<Statement>) -> Self {
        Function {
            identifiers,
            statements,
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "function < {:?} > {{ {:?} }}",
            self.identifiers, self.statements
        )
    }
}
