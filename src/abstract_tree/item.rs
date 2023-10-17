//! Top-level unit of Dust code.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Result, Statement, Value, VariableMap};

/// An abstractiton of an independent unit of source code, or a comment.
///
/// Items are either comments, which do nothing, or statements, which can be run
/// to produce a single value or interact with a context by creating or
/// referencing variables.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Item {
    statements: Vec<Statement>,
}

impl Item {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }

    pub fn run_parallel(&self, source: &str, context: &mut VariableMap) -> Result<Value> {
        let statements = &self.statements;
        let run_result = statements.into_par_iter().try_for_each(|statement| {
            let mut context = context.clone();
            statement.run(source, &mut context).map(|_| ())
        });

        match run_result {
            Ok(()) => Ok(Value::Empty),
            Err(error) => Err(error),
        }
    }
}

impl AbstractTree for Item {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        debug_assert_eq!("item", node.kind());

        let child_count = node.child_count();
        let mut statements = Vec::with_capacity(child_count);

        for index in 0..child_count {
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

        Ok(Item { statements })
    }

    fn run(&self, source: &str, context: &mut VariableMap) -> Result<Value> {
        let mut prev_result = Ok(Value::Empty);

        for statement in &self.statements {
            prev_result?;
            prev_result = statement.run(source, context);
        }

        prev_result
    }
}
