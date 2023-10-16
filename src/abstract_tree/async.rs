use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Item};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Async {
    item: Item,
}

impl AbstractTree for Async {
    fn from_syntax_node(source: &str, node: tree_sitter::Node) -> crate::Result<Self> {
        debug_assert_eq!("async", node.kind());

        let item_node = node.child(2).unwrap();
        let item = Item::from_syntax_node(source, item_node)?;

        Ok(Async { item })
    }

    fn run(&self, source: &str, context: &mut crate::VariableMap) -> crate::Result<crate::Value> {
        self.item.run_parallel(source, context)
    }
}
