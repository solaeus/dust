use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Format, Identifier, Map, Type, TypeSpecification, Value, ValueNode,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct New {
    identifier: Identifier,
    properties: Vec<(Identifier, ValueNode, Option<TypeSpecification>)>,
}

impl AbstractTree for New {
    fn from_syntax(node: Node, source: &str, context: &Map) -> Result<Self, SyntaxError> {
        let identifier_node = node.child(1).unwrap();
        let identifier = Identifier::from_syntax(identifier_node, source, context)?;

        let properties = Vec::new();

        Ok(New {
            identifier,
            properties,
        })
    }

    fn expected_type(&self, _context: &Map) -> Result<Type, ValidationError> {
        todo!()
    }

    fn check_type(&self, _source: &str, _context: &Map) -> Result<(), ValidationError> {
        todo!()
    }

    fn run(&self, _source: &str, _context: &Map) -> Result<Value, RuntimeError> {
        todo!()
    }
}

impl Format for New {
    fn format(&self, _output: &mut String, _indent_level: u8) {
        todo!()
    }
}
