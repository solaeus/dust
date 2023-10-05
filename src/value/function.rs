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

impl EvaluatorTree for Function {
    fn from_syntax_node(node: tree_sitter::Node, source: &str) -> Result<Self> {
        debug_assert_eq!(node.kind(), "function");

        Ok(Function {
            identifiers: vec![],
            statements: vec![],
        })
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
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

#[cfg(test)]
mod tests {
    use crate::{evaluate, Expression};

    use super::*;

    #[test]
    fn evaluate_function() {
        let function = Function::new(
            vec![Identifier::new("output".to_string())],
            vec![Statement::Expression(Expression::Identifier(
                Identifier::new("output".to_string()),
            ))],
        );

        assert_eq!(
            evaluate("function <output> { output }"),
            vec![Ok(Value::Function(function))]
        );
    }
}
