use serde::{Deserialize, Serialize};
use tree_sitter::Node as SyntaxNode;

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, Format, Identifier, Type, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct EnumPattern {
    name: Identifier,
    variant: Identifier,
    inner_identifier: Option<Identifier>,
}

impl EnumPattern {
    pub fn name(&self) -> &Identifier {
        &self.name
    }

    pub fn variant(&self) -> &Identifier {
        &self.variant
    }

    pub fn inner_identifier(&self) -> &Option<Identifier> {
        &self.inner_identifier
    }
}

impl AbstractTree for EnumPattern {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(source, "enum_pattern", node)?;

        let enum_name_node = node.child(0).unwrap();
        let name = Identifier::from_syntax(enum_name_node, source, context)?;

        let enum_variant_node = node.child(2).unwrap();
        let variant = Identifier::from_syntax(enum_variant_node, source, context)?;

        let inner_identifier = if let Some(child) = node.child(4) {
            Some(Identifier::from_syntax(child, source, context)?)
        } else {
            None
        };

        Ok(EnumPattern {
            name,
            variant,
            inner_identifier,
        })
    }

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _source: &str, _context: &Context) -> Result<(), ValidationError> {
        Ok(())
    }

    fn run(&self, _source: &str, _context: &Context) -> Result<Value, RuntimeError> {
        Ok(Value::none())
    }
}

impl Format for EnumPattern {
    fn format(&self, _output: &mut String, _indent_level: u8) {
        todo!()
    }
}
