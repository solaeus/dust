use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Map, Result, Type, Value};

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
        Error::expect_syntax_node(source, "identifier", node)?;

        let identifier = &source[node.byte_range()];

        Ok(Identifier(identifier.to_string()))
    }

    fn run(&self, _source: &str, context: &Map) -> Result<Value> {
        if let Some(value) = context.variables()?.get(&self.0) {
            Ok(value.clone())
        } else {
            Err(Error::VariableIdentifierNotFound(self.inner().clone()))
        }
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        if let Some(value) = context.variables()?.get(&self.0) {
            Ok(value.r#type(context)?)
        } else {
            Ok(Type::Empty)
        }
    }
}
