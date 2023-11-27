use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Type {
    Any,
    Boolean,
    Float,
    Function,
    Integer,
    List,
    Map,
    String,
    Table,
}

impl AbstractTree for Type {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        Error::expect_syntax_node(source, "type", node)?;

        let range_without_punctuation = node.start_byte() + 1..node.end_byte() - 1;

        let r#type = match &source[range_without_punctuation] {
            "any" => Type::Any,
            "bool" => Type::Boolean,
            "float" => Type::Float,
            "fn" => Type::Function,
            "int" => Type::Integer,
            "list" => Type::List,
            "map" => Type::Map,
            "string" => Type::String,
            "table" => Type::Table,
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "bool, fn, int, list, map, string or table",
                    actual: node.kind(),
                    location: node.start_position(),
                    relevant_source: source[node.byte_range()].to_string(),
                })
            }
        };

        Ok(r#type)
    }

    fn run(&self, _source: &str, _context: &mut Map) -> Result<Value> {
        Ok(Value::Empty)
    }
}
