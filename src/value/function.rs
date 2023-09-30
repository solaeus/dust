use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Result, Statement, Value, VariableMap};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Function {
    identifiers: Vec<String>,
    statements: Vec<Statement>,
}

impl Function {
    pub fn new(identifiers: Vec<String>, statements: Vec<Statement>) -> Self {
        Function {
            identifiers,
            statements,
        }
    }

    pub fn run(&self) -> Result<Value> {
        todo!()
    }

    pub fn run_with_context(&self, context: &mut VariableMap) -> Result<Value> {
        todo!()
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
