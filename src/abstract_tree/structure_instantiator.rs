use std::collections::{btree_map, BTreeMap};

use serde::{Deserialize, Serialize};

use crate::{
    AbstractTree, Error, Identifier, Result, Statement, Structure, Type, TypeDefinition, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct StructureInstantiator(BTreeMap<String, (Option<Statement>, Option<TypeDefinition>)>);

impl StructureInstantiator {
    pub fn new() -> Self {
        StructureInstantiator(BTreeMap::new())
    }

    pub fn get(&self, key: &str) -> Option<&(Option<Statement>, Option<TypeDefinition>)> {
        self.0.get(key)
    }

    pub fn set(
        &mut self,
        key: String,
        statement: Option<Statement>,
        type_definition: Option<TypeDefinition>,
    ) {
        self.0.insert(key, (statement, type_definition));
    }

    pub fn iter(&self) -> btree_map::Iter<'_, String, (Option<Statement>, Option<TypeDefinition>)> {
        self.0.iter()
    }
}

impl AbstractTree for StructureInstantiator {
    fn from_syntax_node(
        source: &str,
        node: tree_sitter::Node,
        context: &Structure,
    ) -> crate::Result<Self> {
        Error::expect_syntax_node(source, "structure_instantiator", node)?;

        let child_count = node.child_count();
        let mut instantiator = StructureInstantiator::new();
        let mut current_key = "".to_string();
        let mut current_type = None;

        for index in 1..child_count - 1 {
            let child_syntax_node = node.child(index).unwrap();

            if child_syntax_node.kind() == "identifier" {
                if let Some(type_definition) = current_type {
                    instantiator.set(current_key, None, Some(type_definition));

                    current_type = None;
                }

                current_key =
                    Identifier::from_syntax_node(source, child_syntax_node, context)?.take_inner();
            }

            if child_syntax_node.kind() == "type_definition" {
                current_type = Some(TypeDefinition::from_syntax_node(
                    source,
                    child_syntax_node,
                    context,
                )?);
            }

            if child_syntax_node.kind() == "statement" {
                let statement = Statement::from_syntax_node(source, child_syntax_node, context)?;

                instantiator.set(current_key.clone(), Some(statement), current_type.clone());

                current_type = None;
            }
        }

        Ok(instantiator)
    }

    fn run(&self, source: &str, context: &Structure) -> Result<Value> {
        let mut variables = BTreeMap::new();

        for (key, (statement_option, type_definition_option)) in &self.0 {
            let (value, r#type) = match (statement_option, type_definition_option) {
                (Some(statement), Some(type_definition)) => {
                    let value = statement.run(source, context)?;

                    (value, type_definition.inner().clone())
                }
                (Some(statement), None) => {
                    let value = statement.run(source, context)?;
                    let r#type = value.r#type();

                    (value, r#type)
                }
                (None, Some(r#type)) => (Value::none(), r#type.inner().clone()),
                (None, None) => (Value::none(), Type::None),
            };

            variables.insert(key.clone(), (value, r#type));
        }

        Ok(Value::Structure(Structure::new(variables, self.clone())))
    }

    fn expected_type(&self, _context: &Structure) -> Result<Type> {
        todo!()
    }
}
