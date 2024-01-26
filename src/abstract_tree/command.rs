use std::process;

use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Error, Format, Map, Result, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Command {
    command_text: String,
    command_arguments: Vec<String>,
}

impl AbstractTree for Command {
    fn from_syntax(node: tree_sitter::Node, source: &str, _context: &crate::Map) -> Result<Self> {
        Error::expect_syntax_node(source, "command", node)?;

        let command_text_node = node.child(1).unwrap();
        let command_text = source[command_text_node.byte_range()].to_string();

        let mut command_arguments = Vec::new();

        for index in 2..node.child_count() {
            let text_node = node.child(index).unwrap();
            let mut text = source[text_node.byte_range()].to_string();

            if (text.starts_with('\'') && text.ends_with('\''))
                || (text.starts_with('"') && text.ends_with('"'))
                || (text.starts_with('`') && text.ends_with('`'))
            {
                text = text[1..text.len() - 1].to_string();
            }

            command_arguments.push(text);
        }

        Ok(Command {
            command_text,
            command_arguments,
        })
    }

    fn run(&self, _source: &str, _context: &Map) -> Result<Value> {
        let output = process::Command::new(&self.command_text)
            .args(&self.command_arguments)
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
