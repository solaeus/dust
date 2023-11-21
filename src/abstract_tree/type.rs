use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Type {
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

        let r#type = match &source[node.byte_range()] {
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
        match self {
            Type::Boolean => Ok(Value::String("bool".to_string())),
            Type::Float => Ok(Value::String("float".to_string())),
            Type::Function => Ok(Value::String("fn".to_string())),
            Type::Integer => Ok(Value::String("int".to_string())),
            Type::List => Ok(Value::String("list".to_string())),
            Type::Map => Ok(Value::String("map".to_string())),
            Type::String => Ok(Value::String("string".to_string())),
            Type::Table => Ok(Value::String("table".to_string())),
        }
    }
}
