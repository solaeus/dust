use serde::{Deserialize, Serialize};
use tree_sitter::Node as SyntaxNode;

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, EnumInstance, Format, Identifier, Type, TypeDefinition, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct EnumDefinition {
    identifier: Identifier,
    variants: Vec<(Identifier, Vec<Type>)>,
}

impl EnumDefinition {
    pub fn new(identifier: Identifier, variants: Vec<(Identifier, Vec<Type>)>) -> Self {
        Self {
            identifier,
            variants,
        }
    }

    pub fn instantiate(&self, variant: Identifier, content: Option<Value>) -> EnumInstance {
        EnumInstance::new(self.identifier.clone(), variant, content)
    }

    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    pub fn variants(&self) -> &Vec<(Identifier, Vec<Type>)> {
        &self.variants
    }
}

impl AbstractTree for EnumDefinition {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node("enum_definition", node)?;

        let identifier_node = node.child(1).unwrap();
        let identifier = Identifier::from_syntax(identifier_node, source, context)?;

        let mut variants = Vec::new();
        let mut current_identifier: Option<Identifier> = None;
        let mut types = Vec::new();

        for index in 3..node.child_count() - 1 {
            let child = node.child(index).unwrap();

            if child.kind() == "identifier" {
                if let Some(identifier) = &current_identifier {
                    variants.push((identifier.clone(), types));
                }

                current_identifier = Some(Identifier::from_syntax(child, source, context)?);
                types = Vec::new();
            }

            if child.kind() == "type" {
                let r#type = Type::from_syntax(child, source, context)?;

                types.push(r#type);
            }
        }

        Ok(EnumDefinition {
            identifier,
            variants,
        })
    }

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _source: &str, context: &Context) -> Result<(), ValidationError> {
        context.set_definition(self.identifier.clone(), TypeDefinition::Enum(self.clone()))?;
        self.identifier.validate(_source, context)?;

        Ok(())
    }

    fn run(&self, _source: &str, _context: &Context) -> Result<Value, RuntimeError> {
        Ok(Value::none())
    }
}

impl Format for EnumDefinition {
    fn format(&self, _output: &mut String, _indent_level: u8) {
        todo!()
    }
}
