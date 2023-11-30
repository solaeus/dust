use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Expression, FunctionCall, Map, Result, TypeDefinition, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Yield {
    call: FunctionCall,
}

impl AbstractTree for Yield {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        let input_node = node.child(0).unwrap();
        let input = Expression::from_syntax_node(source, input_node, context)?;

        let function_node = node.child(3).unwrap();
        let function = Expression::from_syntax_node(source, function_node, context)?;

        let mut arguments = Vec::new();

        arguments.push(input);

        for index in 4..node.child_count() - 1 {
            let node = node.child(index).unwrap();

            if node.is_named() {
                let expression = Expression::from_syntax_node(source, node, context)?;

                arguments.push(expression);
            }
        }

        let call = FunctionCall::new(function, arguments);

        Ok(Yield { call })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        self.call.run(source, context)
    }

    fn expected_type(&self, context: &Map) -> Result<TypeDefinition> {
        self.call.expected_type(context)
    }
}
