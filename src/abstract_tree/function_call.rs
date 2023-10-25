use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Map, Result, Value};

use super::{expression::Expression, identifier::Identifier};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionCall {
    name: Identifier,
    arguments: Vec<Expression>,
}

impl AbstractTree for FunctionCall {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        debug_assert_eq!("function_call", node.kind());

        let name_node = node.child(1).unwrap();
        let name = Identifier::from_syntax_node(source, name_node)?;

        let mut arguments = Vec::new();

        for index in 2..node.child_count() - 1 {
            let child = node.child(index).unwrap();

            if child.is_named() {
                let expression = Expression::from_syntax_node(source, child)?;

                arguments.push(expression);
            }
        }

        Ok(FunctionCall { name, arguments })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let key = self.name.inner();
        let definition = if let Some(value) = context.get_value(key)? {
            value.as_function().cloned()?
        } else {
            return Err(Error::FunctionIdentifierNotFound(self.name.clone()));
        };
        let id_expr_pairs = definition.identifiers().iter().zip(self.arguments.iter());
        let mut function_context = context.clone();

        for (identifier, expression) in id_expr_pairs {
            let key = identifier.inner().clone();
            let value = expression.run(source, context)?;

            function_context.set_value(key, value)?;
        }

        definition.body().run(source, &mut function_context)
    }
}
