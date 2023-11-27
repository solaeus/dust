use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Block, Error, Identifier, Map, Result, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Function {
    parameters: Vec<(Identifier, Type)>,
    return_type: Option<Type>,
    body: Block,
}

impl Function {
    pub fn parameters(&self) -> &Vec<(Identifier, Type)> {
        &self.parameters
    }
}

impl AbstractTree for Function {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let mut parameters = Vec::new();
        let mut previous_identifier = None;

        for index in 1..node.child_count() - 2 {
            let child = node.child(index).unwrap();

            if child.kind() == "identifier" {
                previous_identifier = Some(Identifier::from_syntax_node(source, child)?);
            }

            if child.kind() == "type" {
                let identifier = previous_identifier.take().unwrap();
                let r#type = Type::from_syntax_node(source, child)?;

                parameters.push((identifier, r#type))
            }
        }

        let return_type_node = node.child_by_field_name("return_type");
        let return_type = if let Some(child) = return_type_node {
            Some(Type::from_syntax_node(source, child)?)
        } else {
            None
        };

        let body_node = node.child_by_field_name("body").unwrap();
        let body = Block::from_syntax_node(source, body_node)?;

        Ok(Function {
            parameters,
            return_type,
            body,
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let return_value = self.body.run(source, context)?;

        if let Some(r#type) = &self.return_type {
            r#type.check(&return_value)?;
        } else if !return_value.is_empty() {
            return Err(Error::ExpectedEmpty {
                actual: return_value.clone(),
            });
        }

        Ok(return_value)
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Function {{ parameters: {:?}, return_type: {:?}, body: {:?} }}",
            self.parameters, self.return_type, self.body
        )
    }
}
