use serde::{Deserialize, Serialize};
use tree_sitter::Node as SyntaxNode;

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, Format, Type, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct StructDefinition;

impl AbstractTree for StructDefinition {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        todo!()
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

impl Format for StructDefinition {
    fn format(&self, output: &mut String, indent_level: u8) {
        todo!()
    }
}
