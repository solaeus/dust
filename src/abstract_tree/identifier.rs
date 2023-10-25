use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Identifier(String);

impl Identifier {
    pub fn new(inner: String) -> Self {
        Identifier(inner)
    }

    pub fn take_inner(self) -> String {
        self.0
    }

    pub fn inner(&self) -> &String {
        &self.0
    }
}

impl AbstractTree for Identifier {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        debug_assert_eq!("identifier", node.kind());

        let identifier = &source[node.byte_range()];

        Ok(Identifier(identifier.to_string()))
    }

    fn run(&self, _source: &str, context: &mut Map) -> Result<Value> {
        let value = if let Some(value) = context.get_value(&self.0)? {
            value
        } else {
            return Err(Error::VariableIdentifierNotFound(self.inner().clone()));
        };

        Ok(value)
    }
}
