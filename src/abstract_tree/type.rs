use std::fmt::{self, Display, Formatter};

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

impl Type {
    pub fn check(&self, value: &Value) -> Result<()> {
        match (self, value.r#type()) {
            (Type::Any, _)
            | (Type::Boolean, Type::Boolean)
            | (Type::Float, Type::Float)
            | (Type::Function, Type::Function)
            | (Type::Integer, Type::Integer)
            | (Type::List, Type::List)
            | (Type::Map, Type::Map)
            | (Type::String, Type::String)
            | (Type::Table, Type::Table) => Ok(()),
            (Type::Boolean, _) => Err(Error::ExpectedBoolean {
                actual: value.clone(),
            }),
            (Type::Float, _) => Err(Error::ExpectedFloat {
                actual: value.clone(),
            }),
            (Type::Function, _) => Err(Error::ExpectedFunction {
                actual: value.clone(),
            }),
            (Type::Integer, _) => Err(Error::ExpectedInteger {
                actual: value.clone(),
            }),
            (Type::List, _) => Err(Error::ExpectedList {
                actual: value.clone(),
            }),
            (Type::Map, _) => Err(Error::ExpectedMap {
                actual: value.clone(),
            }),
            (Type::String, _) => Err(Error::ExpectedString {
                actual: value.clone(),
            }),
            (Type::Table, _) => Err(Error::ExpectedTable {
                actual: value.clone(),
            }),
        }
    }
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
            "str" => Type::String,
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

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Type::Any => write!(f, "any"),
            Type::Boolean => write!(f, "bool"),
            Type::Float => write!(f, "float"),
            Type::Function => write!(f, "function"),
            Type::Integer => write!(f, "integer"),
            Type::List => write!(f, "list"),
            Type::Map => write!(f, "map"),
            Type::String => write!(f, "string"),
            Type::Table => write!(f, "table"),
        }
    }
}
