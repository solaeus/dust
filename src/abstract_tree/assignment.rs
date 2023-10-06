use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Result, Value, VariableMap};

use super::{identifier::Identifier, statement::Statement};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Assignment {
    identifier: Identifier,
    statement: Statement,
}

impl AbstractTree for Assignment {
    fn from_syntax_node(node: Node, source: &str) -> Result<Self> {
        let identifier_node = node.child(0).unwrap();
        let identifier = Identifier::from_syntax_node(identifier_node, source)?;

        let statement_node = node.child(2).unwrap();
        let statement = Statement::from_syntax_node(statement_node, source)?;

        Ok(Assignment {
            identifier,
            statement,
        })
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        let key = self.identifier.clone().take_inner();
        let value = self.statement.run(context)?;

        context.set_value(key, value)?;

        Ok(Value::Empty)
    }
}
