use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Result, Value, VariableMap};

use super::{expression::Expression, identifier::Identifier};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionCall {
    identifier: Identifier,
    expressions: Vec<Expression>,
}

impl AbstractTree for FunctionCall {
    fn from_syntax_node(node: Node, source: &str) -> Result<Self> {
        debug_assert_eq!("function_call", node.kind());

        let identifier_node = node.child(1).unwrap();
        let identifier = Identifier::from_syntax_node(identifier_node, source)?;

        let mut expressions = Vec::new();

        let mut current_index = 2;
        while current_index < node.child_count() - 1 {
            let expression_node = node.child(current_index).unwrap();
            let expression = Expression::from_syntax_node(expression_node, source)?;

            expressions.push(expression);

            current_index += 1;
        }

        Ok(FunctionCall {
            identifier,
            expressions,
        })
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        let identifier = &self.identifier;
        let definition = if let Some(value) = context.get_value(identifier.inner())? {
            value
        } else {
            return Err(crate::Error::FunctionIdentifierNotFound(identifier.clone()));
        };
        let mut arguments = Vec::with_capacity(self.expressions.len());

        Ok(Value::List(arguments))
    }
}
