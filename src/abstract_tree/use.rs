use std::fs::read_to_string;

use serde::{Deserialize, Serialize};

use crate::{evaluate_with_context, AbstractTree, Error, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Use {
    path: String,
}

impl AbstractTree for Use {
    fn from_syntax_node(source: &str, node: tree_sitter::Node) -> crate::Result<Self> {
        Error::expect_syntax_node(source, "use", node)?;

        let string_node = node.child(1).unwrap();
        let path = source[string_node.start_byte() + 1..string_node.end_byte() - 1].to_string();

        println!("{path}");

        Ok(Use { path })
    }

    fn run(&self, _source: &str, _context: &mut Map) -> Result<Value> {
        let file_contents = read_to_string(&self.path)?;
        let mut file_context = Map::new();

        evaluate_with_context(&file_contents, &mut file_context)?;

        Ok(Value::Map(file_context))
    }
}
