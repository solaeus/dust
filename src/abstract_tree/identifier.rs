use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use serde::{de::Visitor, Deserialize, Serialize};

use crate::{
    built_in_identifiers::all_built_in_identifiers,
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, Format, SyntaxNode, Type, Value,
};

/// A string by which a variable is known to a context.
///
/// Every variable is a key-value pair. An identifier holds the key part of that
/// pair. Its inner value can be used to retrieve a Value instance from a Map.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Identifier(Arc<String>);

impl Identifier {
    pub fn new(key: &str) -> Self {
        for built_in_identifier in all_built_in_identifiers() {
            let identifier = built_in_identifier.get();

            if &key == identifier.inner().as_ref() {
                return identifier.clone();
            }
        }

        Identifier(Arc::new(key.to_string()))
    }

    pub fn from_raw_parts(arc: Arc<String>) -> Self {
        Identifier(arc)
    }

    pub fn inner(&self) -> &Arc<String> {
        &self.0
    }

    pub fn contains(&self, string: &str) -> bool {
        self.0.as_ref() == string
    }
}

impl AbstractTree for Identifier {
    fn from_syntax(
        node: SyntaxNode,
        source: &str,
        _context: &Context,
    ) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(source, "identifier", node)?;

        let text = &source[node.byte_range()];

        debug_assert!(!text.is_empty());

        Ok(Identifier::new(text))
    }

    fn validate(&self, _source: &str, _context: &Context) -> Result<(), ValidationError> {
        Ok(())
    }

    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError> {
        if let Some(r#type) = context.get_type(self)? {
            Ok(r#type)
        } else {
            Err(ValidationError::VariableIdentifierNotFound(self.clone()))
        }
    }

    fn run(&self, _source: &str, context: &Context) -> Result<Value, RuntimeError> {
        if let Some(value) = context.get_value(self)? {
            Ok(value.clone())
        } else {
            Err(RuntimeError::VariableIdentifierNotFound(self.clone()))
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

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.as_ref())
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_string(IdentifierVisitor)
    }
}

struct IdentifierVisitor;

impl<'de> Visitor<'de> for IdentifierVisitor {
    type Value = Identifier;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("valid UFT-8 sequence")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Identifier(Arc::new(v)))
    }
}
