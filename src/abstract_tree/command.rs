use std::process;

use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Error, Format, Map, Result, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Command {
    command_texts: Vec<String>,
}

impl AbstractTree for Command {
    fn from_syntax(node: tree_sitter::Node, source: &str, _context: &crate::Map) -> Result<Self> {
        Error::expect_syntax_node(source, "command", node)?;

        let mut command_texts = Vec::new();

        for index in 1..node.child_count() {
            let text_node = node.child(index).unwrap();
            let text = source[text_node.byte_range()].to_string();

            command_texts.push(text);
        }

        Ok(Command { command_texts })
    }

    fn run(&self, _source: &str, _context: &Map) -> Result<Value> {
        let output = process::Command::new(self.command_texts.first().unwrap())
            .args(&self.command_texts[1..])
            .spawn()?
            .wait_with_output()?
            .stdout;
        let string = String::from_utf8(output)?;

        Ok(Value::String(string))
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        Ok(Type::String)
    }
}

impl Format for Command {
    fn format(&self, _output: &mut String, _indent_level: u8) {
        todo!()
    }
}
