use std::sync::RwLock;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Map, Result, Statement, Value};

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

        let statement_count = if is_async {
            node.child_count() - 3
        } else {
            node.child_count() - 2
        };
        let mut statements = Vec::with_capacity(statement_count);

        for index in 1..node.child_count() - 1 {
            let child_node = node.child(index).unwrap();

            if child_node.is_named() {
                let statement = Statement::from_syntax_node(source, child_node)?;
                statements.push(statement);
            }
        }

        Ok(Block {
            is_async,
            statements,
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        if self.is_async {
            let statements = &self.statements;
            let final_result = RwLock::new(Ok(Value::Empty));

            statements
                .into_par_iter()
                .enumerate()
                .find_map_first(|(index, statement)| {
                    if let Statement::Return(expression) = statement {
                        return Some(expression.run(source, &mut context.clone()));
                    }

                    let result = statement.run(source, &mut context.clone());

                    if result.is_err() {
                        Some(result)
                    } else if index == statements.len() - 1 {
                        let _ = final_result.write().unwrap().as_mut().map(|_| result);

                        None
                    } else {
                        None
                    }
                })
                .unwrap_or(final_result.into_inner().unwrap())
        } else {
            let mut prev_result = None;

            for statement in &self.statements {
                if let Statement::Return(expression) = statement {
                    return expression.run(source, context);
                }

                prev_result = Some(statement.run(source, context));
            }

            prev_result.unwrap_or(Ok(Value::Empty))
        }
    }
}
