use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Format, Identifier, Type, TypeSpecification, Value, ValueNode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct New {
    identifier: Identifier,
    properties: Vec<(Identifier, ValueNode, Option<TypeSpecification>)>,
}

impl AbstractTree for New {
    fn from_syntax(
        node: tree_sitter::Node,
        source: &str,
        context: &crate::Map,
    ) -> crate::Result<Self> {
        todo!()
    }

    fn run(&self, source: &str, context: &crate::Map) -> crate::Result<Value> {
        todo!()
    }

    fn expected_type(&self, context: &crate::Map) -> crate::Result<Type> {
        todo!()
    }
}

impl Format for New {
    fn format(&self, output: &mut String, indent_level: u8) {
        todo!()
    }
}
