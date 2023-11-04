use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Block, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Async {
    block: Block,
}

impl AbstractTree for Async {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        debug_assert_eq!("async", node.kind());

        let block_node = node.child(1).unwrap();
        let block = Block::from_syntax_node(source, block_node)?;

        Ok(Async { block })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let statements = self.block.statements();

        statements
            .into_par_iter()
            .enumerate()
            .find_map_first(|(index, statement)| {
                let mut context = context.clone();
                let result = statement.run(source, &mut context);

                result.clone().unwrap();
                if result.is_err() {
                    Some(result)
                } else if index == statements.len() - 1 {
                    Some(result)
                } else {
                    None
                }
            })
            .unwrap_or(Ok(Value::Empty))
    }
}
