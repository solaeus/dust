use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tree_sitter::Node as SyntaxNode;

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, Format, Identifier, Map, SourcePosition, Statement, Type,
    TypeSpecification, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct MapNode {
    properties: BTreeMap<Identifier, (Statement, Option<Type>)>,
    position: SourcePosition,
}

impl MapNode {
    pub fn properties(&self) -> &BTreeMap<Identifier, (Statement, Option<Type>)> {
        &self.properties
    }
}

impl AbstractTree for MapNode {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node("map", node)?;

        let mut properties = BTreeMap::new();
        let mut current_identifier = None;
        let mut current_type = None;

        for index in 0..node.child_count() - 1 {
            let child = node.child(index).unwrap();

            if child.kind() == "identifier" {
                current_identifier = Some(Identifier::from_syntax(child, source, context)?);
                current_type = None;
            }

            if child.kind() == "type_specification" {
                current_type =
                    Some(TypeSpecification::from_syntax(child, source, context)?.take_inner());
            }

            if child.kind() == "statement" {
                let statement = Statement::from_syntax(child, source, context)?;

                if let Some(identifier) = &current_identifier {
                    properties.insert(identifier.clone(), (statement, current_type.clone()));
                }
            }
        }

        Ok(MapNode {
            properties,
            position: SourcePosition::from(node.range()),
        })
    }

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::Map)
    }

    fn validate(&self, _source: &str, context: &Context) -> Result<(), ValidationError> {
        for (_key, (statement, r#type)) in &self.properties {
            statement.validate(_source, context)?;

            if let Some(expected) = r#type {
                let actual = statement.expected_type(context)?;

                if !expected.accepts(&actual) {
                    return Err(ValidationError::TypeCheck {
                        expected: expected.clone(),
                        actual,
                        position: self.position.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    fn run(&self, _source: &str, _context: &Context) -> Result<Value, RuntimeError> {
        let mut map = Map::new();

        for (key, (statement, _)) in &self.properties {
            let value = statement.run(_source, _context)?;

            map.set(key.clone(), value);
        }

        Ok(Value::Map(map))
    }
}

impl Format for MapNode {
    fn format(&self, _output: &mut String, _indent_level: u8) {
        todo!()
    }
}
