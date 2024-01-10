use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Error, Format, Map, Result, SyntaxNode, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct TypeDefinition {
    r#type: Type,
}

impl TypeDefinition {
    pub fn new(r#type: Type) -> Self {
        Self { r#type }
    }

    pub fn inner(&self) -> &Type {
        &self.r#type
    }

    pub fn take_inner(self) -> Type {
        self.r#type
    }
}

impl AbstractTree for TypeDefinition {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "type_definition", node)?;

        let type_node = node.child(1).unwrap();
        let r#type = Type::from_syntax(type_node, source, context)?;

        Ok(TypeDefinition { r#type })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        self.r#type.run(source, context)
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        self.r#type.expected_type(context)
    }
}

impl Format for TypeDefinition {
    fn format(&self, output: &mut String, indent_level: u8) {
        output.push('<');
        self.r#type.format(output, indent_level);
        output.push('>');
    }
}
