use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Error, Format, Identifier, Map, Result, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Command {
    binary_name: String,
    arguments: Vec<String>,
}

impl AbstractTree for Command {
    fn from_syntax(node: tree_sitter::Node, source: &str, context: &crate::Map) -> Result<Self> {
        Error::expect_syntax_node(source, "command", node)?;
        let identifier_node = node.child(1)
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        todo!()
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        todo!()
    }
}

impl Format for Command {
    fn format(&self, output: &mut String, indent_level: u8) {
        todo!()
    }
}
