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
        Error::expect_syntax_node(source, "function", node)?;

        let child_count = node.child_count();
        let mut parameters = Vec::new();

        for index in 1..child_count - 2 {
            let parameter_node = {
                let child = node.child(index).unwrap();

                if child.is_named() {
                    child
                } else {
                    continue;
                }
            };

            Error::expect_syntax_node(source, "parameter", parameter_node)?;

            let identifier_node = parameter_node.child(0).unwrap();
            let identifier = Identifier::from_syntax_node(source, identifier_node)?;

            let type_node = parameter_node.child(1).unwrap();
            let r#type = Type::from_syntax_node(source, type_node)?;

            parameters.push((identifier, r#type))
        }

        let return_type_node = node.child(child_count - 2).unwrap();
        let return_type = if return_type_node.is_named() {
            Some(Type::from_syntax_node(source, return_type_node)?)
        } else {
            None
        };

        let body_node = node.child(child_count - 1).unwrap();
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
