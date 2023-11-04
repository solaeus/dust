use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Result, Statement};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Block {
    statements: Vec<Statement>,
}

impl Block {
    pub fn statements(&self) -> &Vec<Statement> {
        &self.statements
    }
}

impl AbstractTree for Block {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        debug_assert_eq!("block", node.kind());

        let statement_count = node.child_count();
        let mut statements = Vec::with_capacity(statement_count);

        for index in 0..statement_count {
            let child_node = node.child(index).unwrap();

            if child_node.kind() == "statement" {
                let statement = Statement::from_syntax_node(source, child_node)?;
                statements.push(statement);
            }
        }

        Ok(Block { statements })
    }

    fn run(&self, source: &str, context: &mut crate::Map) -> crate::Result<crate::Value> {
        for statement in &self.statements[0..self.statements.len() - 1] {
            statement.run(source, context)?;
        }

        let final_statement = self.statements.last().unwrap();
        let final_value = final_statement.run(source, context)?;

        Ok(final_value)
    }
}
