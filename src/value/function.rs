use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Block, Error, Identifier, Map, Result, TypeDefinition, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Function {
    parameters: Vec<Identifier>,
    body: Block,
}

impl Function {
    pub fn parameters(&self) -> &Vec<Identifier> {
        &self.parameters
    }

    pub fn body(&self) -> &Block {
        &self.body
    }
}

impl AbstractTree for Function {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "function", node)?;

        let child_count = node.child_count();
        let mut parameters = Vec::new();

        for index in 1..child_count - 2 {
            let child = node.child(index).unwrap();

            if child.is_named() {
                let identifier = Identifier::from_syntax_node(source, child, context)?;
                parameters.push(identifier);
            }
        }

        let body_node = node.child(child_count - 1).unwrap();
        let body = Block::from_syntax_node(source, body_node, context)?;

        Ok(Function { parameters, body })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let return_value = self.body.run(source, context)?;

        Ok(return_value)
    }

    fn expected_type(&self, context: &Map) -> Result<TypeDefinition> {
        self.body.expected_type(context)
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Function {{ parameters: {:?}, body: {:?} }}",
            self.parameters, self.body
        )
    }
}
