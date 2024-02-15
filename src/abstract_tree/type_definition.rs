use serde::{Deserialize, Serialize};
use tree_sitter::Node as SyntaxNode;

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, EnumDefinition, Format, StructDefinition, Type, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum TypeDefinition {
    Enum(EnumDefinition),
    Struct(StructDefinition),
}

impl AbstractTree for TypeDefinition {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(source, "type_definition", node)?;

        let child = node.child(0).unwrap();

        match child.kind() {
            "enum_definition" => Ok(TypeDefinition::Enum(EnumDefinition::from_syntax(
                child, source, context,
            )?)),
            "struct_definition" => Ok(TypeDefinition::Struct(StructDefinition::from_syntax(
                child, source, context,
            )?)),
            _ => Err(SyntaxError::UnexpectedSyntaxNode {
                expected: "enum or struct definition".to_string(),
                actual: child.kind().to_string(),
                location: child.start_position(),
                relevant_source: source[child.byte_range()].to_string(),
            }),
        }
    }

    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError> {
        todo!()
    }

    fn validate(&self, source: &str, context: &Context) -> Result<(), ValidationError> {
        todo!()
    }

    fn run(&self, source: &str, context: &Context) -> Result<Value, RuntimeError> {
        todo!()
    }
}

impl Format for TypeDefinition {
    fn format(&self, output: &mut String, indent_level: u8) {
        todo!()
    }
}
