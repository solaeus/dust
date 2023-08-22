use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{eval, eval_with_context, Result, Value, VariableMap};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Function(String);

impl Function {
    pub fn new(body: &str) -> Self {
        Function(body.to_string())
    }

    pub fn run(&self) -> Result<Value> {
        eval(&self.0)
    }

    pub fn run_with_context(&self, context: &mut VariableMap) -> Result<Value> {
        eval_with_context(&self.0, context)
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
