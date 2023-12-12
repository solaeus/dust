use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Block, Function, Identifier, Map, Result, Type, TypeDefinition, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionDeclaration {
    name: Option<Identifier>,
    r#type: Option<Type>,
    parameters: Vec<Identifier>,
    body: Block,
}

impl AbstractTree for FunctionDeclaration {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        let name_node = node.child_by_field_name("name");
        let name = if let Some(child) = name_node {
            Some(Identifier::from_syntax_node(source, child, context)?)
        } else {
            None
        };

        let type_definition_node = node.child_by_field_name("type");
        let type_definition = if let Some(child) = type_definition_node {
            Some(TypeDefinition::from_syntax_node(source, child, context)?)
        } else {
            None
        };

        let mut parameters = Vec::new();

        if node.child_by_field_name("parameters").is_some() {
            for index in 3..node.child_count() - 2 {
                let child = node.child(index).unwrap();

                if child.is_named() {
                    let parameter = Identifier::from_syntax_node(source, child, context)?;

                    parameters.push(parameter);
                }
            }
        }

        let body_node = node.child_by_field_name("body").unwrap();
        let body = Block::from_syntax_node(source, body_node, context)?;

        Ok(FunctionDeclaration {
            name,
            r#type: type_definition.map(|defintion| defintion.take_inner()),
            parameters,
            body,
        })
    }

    fn run(&self, _source: &str, context: &Map) -> Result<Value> {
        let value = Value::Function(Function::new(
            self.parameters.clone(),
            self.body.clone(),
            self.r#type.clone(),
        ));

        if let Some(name) = &self.name {
            let key = name.inner().clone();

            context.set(key, value, self.r#type.clone())?;

            Ok(Value::Empty)
        } else {
            Ok(value)
        }
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        if self.name.is_some() {
            Ok(Type::Empty)
        } else {
            Ok(self.r#type.clone().unwrap_or(Type::Function {
                parameter_types: vec![Type::Any; self.parameters.len()],
                return_type: Box::new(Type::Any),
            }))
        }
    }
}
