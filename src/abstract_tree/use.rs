use std::fs::read_to_string;

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{interpret_with_context, AbstractTree, Error, Map, Result, Type, Value};

/// Abstract representation of a use statement.
///
/// Use will evaluate the Dust file at the given path. It will create an empty
/// context to do so, then apply every value from that context to the current
/// context.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Use {
    path: String,
}

impl AbstractTree for Use {
    fn from_syntax_node(source: &str, node: Node, _context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "use", node)?;

        let string_node = node.child(1).unwrap();
        let path = source[string_node.start_byte() + 1..string_node.end_byte() - 1].to_string();

        Ok(Use { path })
    }

    fn run(&self, _source: &str, context: &Map) -> Result<Value> {
        let file_contents = read_to_string(&self.path)?;
        let file_context = Map::new();

        interpret_with_context(&file_contents, file_context.clone())?;

        for (key, (value, r#type)) in file_context.variables()?.iter() {
            context.set(key.clone(), value.clone(), Some(r#type.clone()))?;
        }

        Ok(Value::Map(file_context))
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        Ok(Type::Map)
    }
}
