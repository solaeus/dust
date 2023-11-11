use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Result, Statement, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Block {
    is_async: bool,
    statements: Vec<Statement>,
}

impl AbstractTree for Block {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        debug_assert_eq!("block", node.kind());

        let first_child = node.child(0).unwrap();
        let is_async = first_child.kind() == "async";

        let statement_count = node.child_count();
        let mut statements = Vec::with_capacity(statement_count);

        for index in 0..statement_count - 1 {
            let child_node = node.child(index).unwrap();

            if child_node.kind() == "statement" {
                let statement = Statement::from_syntax_node(source, child_node)?;
                statements.push(statement);
            }
        }

        Ok(Block {
            is_async,
            statements,
        })
    }

    fn run(&self, source: &str, context: &mut crate::Map) -> crate::Result<crate::Value> {
        if self.is_async {
            let statements = &self.statements;

            statements
                .into_par_iter()
                .enumerate()
                .find_map_first(|(index, statement)| {
                    let mut context = context.clone();
                    let result = statement.run(source, &mut context);

                    if result.is_err() {
                        Some(result)
                    } else if index == statements.len() - 1 {
                        Some(result)
                    } else {
                        None
                    }
                })
                .unwrap_or(Ok(Value::Empty))
        } else {
            for statement in &self.statements[0..self.statements.len() - 1] {
                statement.run(source, context)?;
            }

            let final_statement = self.statements.last().unwrap();
            let final_value = final_statement.run(source, context)?;

            Ok(final_value)
        }
    }
}
