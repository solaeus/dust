use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Error, Format, Map, SyntaxNode, Type, Value,
};

/// A string by which a variable is known to a context.
///
/// Every variable is a key-value pair. An identifier holds the key part of that
/// pair. Its inner value can be used to retrieve a Value instance from a Map.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Identifier(String);

impl Identifier {
    pub fn new<T: Into<String>>(inner: T) -> Self {
        Identifier(inner.into())
    }

    pub fn take_inner(self) -> String {
        self.0
    }

    pub fn inner(&self) -> &String {
        &self.0
    }
}

impl AbstractTree for Identifier {
    fn from_syntax(node: SyntaxNode, source: &str, _context: &Map) -> Result<Self, SyntaxError> {
        Error::expect_syntax_node(source, "identifier", node)?;

        let text = &source[node.byte_range()];

        debug_assert!(!text.is_empty());

        Ok(Identifier(text.to_string()))
    }

    fn check_type(&self, _source: &str, _context: &Map) -> Result<(), ValidationError> {
        Ok(())
    }

    fn expected_type(&self, context: &Map) -> Result<Type, ValidationError> {
        if let Some((_value, r#type)) = context.variables()?.get(&self.0) {
            Ok(r#type.clone())
        } else {
            Err(ValidationError::VariableIdentifierNotFound(self.clone()))
        }
    }

    fn run(&self, _source: &str, context: &Map) -> Result<Value, RuntimeError> {
        if let Some((value, _)) = context.variables()?.get(&self.0) {
            Ok(value.clone())
        } else {
            Err(RuntimeError::VariableIdentifierNotFound(self.0.clone()))
        }
    }
}

impl Format for Identifier {
    fn format(&self, output: &mut String, _indent_level: u8) {
        output.push_str(&self.0);
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
