use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Map, Result, Type, Value, BUILT_IN_FUNCTIONS};

/// A string by which a variable is known to a context.
///
/// Every variable is a key-value pair. An identifier holds the key part of that
/// pair. Its inner value can be used to retrieve a Value instance from a Map.
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
    fn from_syntax_node(source: &str, node: Node, _context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "identifier", node)?;

        let text = &source[node.byte_range()];

        if text.is_empty() {
            println!("{node:?}");
        }

        debug_assert!(!text.is_empty());

        Ok(Identifier(text.to_string()))
    }

    fn run(&self, _source: &str, context: &Map) -> Result<Value> {
        if let Some((value, _)) = context.variables()?.get(&self.0) {
            Ok(value.clone())
        } else {
            Err(Error::VariableIdentifierNotFound(self.inner().clone()))
        }
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        if let Some((_value, r#type)) = context.variables()?.get(&self.0) {
            Ok(r#type.clone())
        } else {
            for built_in_function in BUILT_IN_FUNCTIONS {
                if self.0 == built_in_function.name() {
                    return Ok(built_in_function.r#type());
                }
            }

            Ok(Type::None)
        }
    }
}
