use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Error, Format, Map, SyntaxNode, Type, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct TypeSpecification {
    r#type: Type,
}

impl TypeSpecification {
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

impl AbstractTree for TypeSpecification {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self, SyntaxError> {
        Error::expect_syntax_node(source, "type_specification", node)?;

        let type_node = node.child(1).unwrap();
        let r#type = Type::from_syntax(type_node, source, context)?;

        Ok(TypeSpecification { r#type })
    }

    fn expected_type(&self, context: &Map) -> Result<Type, ValidationError> {
        self.r#type.expected_type(context)
    }

    fn check_type(&self, _source: &str, _context: &Map) -> Result<(), ValidationError> {
        Ok(())
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value, RuntimeError> {
        self.r#type.run(source, context)
    }
}

impl Format for TypeSpecification {
    fn format(&self, output: &mut String, indent_level: u8) {
        output.push('<');
        self.r#type.format(output, indent_level);
        output.push('>');
    }
}
