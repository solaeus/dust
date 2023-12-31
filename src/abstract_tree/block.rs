use std::sync::RwLock;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Map, Result, Statement, Type, Value};

/// Abstract representation of a block.
///
/// A block is almost identical to the root except that it must have curly
/// braces and can optionally be asynchronous. A block evaluates to the value of
/// its final statement but an async block will short-circuit if a statement
/// results in an error. Note that this will be the first statement to encounter
/// an error at runtime, not necessarilly the first statement as they are
/// written.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Block {
    is_async: bool,
    statements: Vec<Statement>,
}

impl AbstractTree for Block {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "block", node)?;

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
                let statement = Statement::from_syntax_node(source, child_node, context)?;
                statements.push(statement);
            }
        }

        Ok(Block {
            is_async,
            statements,
        })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        if self.is_async {
            let statements = &self.statements;
            let final_result = RwLock::new(Ok(Value::none()));

            statements
                .into_par_iter()
                .enumerate()
                .find_map_first(|(index, statement)| {
                    if let Statement::Return(expression) = statement {
                        return Some(expression.run(source, context));
                    }

                    let result = statement.run(source, context);

                    if result.is_err() {
                        Some(result)
                    } else if index == statements.len() - 1 {
                        let _ = final_result.write().unwrap().as_mut().map(|_| result);

                        None
                    } else {
                        None
                    }
                })
                .unwrap_or(final_result.into_inner()?)
        } else {
            let mut prev_result = None;

            for statement in &self.statements {
                if let Statement::Return(expression) = statement {
                    return expression.run(source, context);
                }

                prev_result = Some(statement.run(source, context));
            }

            prev_result.unwrap_or(Ok(Value::none()))
        }
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        if self.is_async {
            Ok(Type::Any)
        } else {
            self.statements.last().unwrap().expected_type(context)
        }
    }
}
