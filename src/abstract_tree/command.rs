use crate::{AbstractTree, Format, Identifier, Result};

pub struct Command {
    binary_name: String,
    arguments: Vec<String>,
}

impl AbstractTree for Command {
    fn from_syntax(
        node: tree_sitter::Node,
        source: &str,
        context: &crate::Map,
    ) -> crate::Result<Self> {
        todo!()
    }

    fn run(&self, source: &str, context: &crate::Map) -> crate::Result<crate::Value> {
        todo!()
    }

    fn expected_type(&self, context: &crate::Map) -> crate::Result<crate::Type> {
        todo!()
    }
}

impl Format for Command {
    fn format(&self, output: &mut String, indent_level: u8) {
        todo!()
    }
}
