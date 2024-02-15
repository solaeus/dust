use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tree_sitter::Node as SyntaxNode;

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, Format, Identifier, Map, MapNode, Statement, StructInstance, Type,
    TypeDefinition, TypeSpecification, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct StructDefinition {
    name: Identifier,
    properties: BTreeMap<String, (Option<Statement>, Type)>,
}

impl StructDefinition {
    pub fn instantiate(
        &self,
        new_properties: &MapNode,
        source: &str,
        context: &Context,
    ) -> Result<StructInstance, RuntimeError> {
        let mut all_properties = Map::new();

        for (key, (statement_option, _)) in &self.properties {
            if let Some(statement) = statement_option {
                let value = statement.run(source, context)?;

                all_properties.set(key.clone(), value);
            }
        }

        for (key, (statement, _)) in new_properties.properties() {
            let value = statement.run(source, context)?;

            all_properties.set(key.clone(), value);
        }

        Ok(StructInstance::new(
            self.name.inner().clone(),
            all_properties,
        ))
    }
}

impl AbstractTree for StructDefinition {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(source, "struct_definition", node)?;

        let name_node = node.child(1).unwrap();
        let name = Identifier::from_syntax(name_node, source, context)?;

        let mut properties = BTreeMap::new();
        let mut current_identifier: Option<Identifier> = None;
        let mut current_type: Option<Type> = None;
        let mut current_statement = None;

        for index in 2..node.child_count() - 1 {
            let child_syntax_node = node.child(index).unwrap();

            if child_syntax_node.kind() == "identifier" {
                if current_statement.is_none() {
                    if let (Some(identifier), Some(r#type)) = (&current_identifier, &current_type) {
                        properties.insert(identifier.inner().clone(), (None, r#type.clone()));
                    }
                }

                current_type = None;
                current_identifier =
                    Some(Identifier::from_syntax(child_syntax_node, source, context)?);
            }

            if child_syntax_node.kind() == "type_specification" {
                current_type = Some(
                    TypeSpecification::from_syntax(child_syntax_node, source, context)?
                        .take_inner(),
                );
            }

            if child_syntax_node.kind() == "statement" {
                current_statement =
                    Some(Statement::from_syntax(child_syntax_node, source, context)?);

                if let Some(identifier) = &current_identifier {
                    let r#type = if let Some(r#type) = &current_type {
                        r#type.clone()
                    } else {
                        Type::None
                    };

                    properties.insert(
                        identifier.inner().clone(),
                        (current_statement.clone(), r#type.clone()),
                    );
                }
            }
        }

        Ok(StructDefinition { name, properties })
    }

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _source: &str, _context: &Context) -> Result<(), ValidationError> {
        Ok(())
    }

    fn run(&self, _source: &str, context: &Context) -> Result<Value, RuntimeError> {
        context.set_definition(
            self.name.inner().clone(),
            TypeDefinition::Struct(self.clone()),
        )?;

        Ok(Value::none())
    }
}

impl Format for StructDefinition {
    fn format(&self, _output: &mut String, _indent_level: u8) {
        todo!()
    }
}
