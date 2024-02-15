use serde::{Deserialize, Serialize};
use tree_sitter::Node as SyntaxNode;

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, EnumDefinition, Format, Identifier, StructDefinition, Type, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum TypeDefinition {
    Enum(EnumDefinition),
    Struct(StructDefinition),
}

impl TypeDefinition {
    pub fn identifier(&self) -> &Identifier {
        match self {
            TypeDefinition::Enum(enum_definition) => enum_definition.identifier(),
            TypeDefinition::Struct(_) => todo!(),
        }
    }
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

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match self {
            TypeDefinition::Enum(enum_definition) => enum_definition.expected_type(_context),
            TypeDefinition::Struct(struct_definition) => struct_definition.expected_type(_context),
        }
    }

    fn validate(&self, _source: &str, _context: &Context) -> Result<(), ValidationError> {
        match self {
            TypeDefinition::Enum(enum_definition) => enum_definition.validate(_source, _context),
            TypeDefinition::Struct(struct_definition) => {
                struct_definition.validate(_source, _context)
            }
        }
    }

    fn run(&self, _source: &str, _context: &Context) -> Result<Value, RuntimeError> {
        match self {
            TypeDefinition::Enum(enum_definition) => enum_definition.run(_source, _context),
            TypeDefinition::Struct(struct_definition) => struct_definition.run(_source, _context),
        }
    }
}

impl Format for TypeDefinition {
    fn format(&self, _output: &mut String, _indent_level: u8) {
        todo!()
    }
}
