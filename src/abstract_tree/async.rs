use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Map, Result, Statement, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Async {
    statements: Vec<Statement>,
}

impl AbstractTree for Async {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        debug_assert_eq!("async", node.kind());

        let child_count = node.child_count();
        let mut statements = Vec::with_capacity(child_count);

        for index in 2..child_count - 1 {
            let child = node.child(index).unwrap();

            let statement = match child.kind() {
                "statement" => Statement::from_syntax_node(source, child)?,
                _ => {
                    return Err(Error::UnexpectedSyntaxNode {
                        expected: "comment or statement",
                        actual: child.kind(),
                        location: child.start_position(),
                        relevant_source: source[child.byte_range()].to_string(),
                    })
                }
            };

            statements.push(statement);
        }

        Ok(Async { statements })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
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
            .unwrap()
    }
}
