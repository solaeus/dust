use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Expression, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Await {
    expressions: Vec<Expression>,
}

impl AbstractTree for Await {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        debug_assert_eq!("await", node.kind());

        let mut expressions = Vec::new();

        for index in 2..node.child_count() - 1 {
            let child = node.child(index).unwrap();

            if child.is_named() {
                let expression = Expression::from_syntax_node(source, child)?;

                expressions.push(expression);
            }
        }

        Ok(Await { expressions })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let expressions = &self.expressions;

        expressions
            .into_par_iter()
            .find_map_first(|expression| {
                let mut context = context.clone();
                let value = if let Ok(value) = expression.run(source, &mut context) {
                    value
                } else {
                    return None;
                };

                let run_result = match value {
                    Value::Future(block) => block.run(source, &mut context),
                    _ => return None,
                };

                Some(run_result)
            })
            .unwrap_or(Ok(Value::Empty))
    }
}
