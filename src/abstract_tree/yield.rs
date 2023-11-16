use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, BuiltInFunction, Expression, FunctionCall, Identifier, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Yield {
    call: FunctionCall,
}

impl AbstractTree for Yield {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let input_node = node.child(0).unwrap();
        let input = Expression::from_syntax_node(source, input_node)?;

        let function_node = node.child(3).unwrap();
        let mut arguments = Vec::new();

        arguments.push(input);

        for index in 4..node.child_count() - 1 {
            let node = node.child(index).unwrap();

            if node.is_named() {
                let expression = Expression::from_syntax_node(source, node)?;

                arguments.push(expression);
            }
        }

        let call = if function_node.kind() == "built_in_function" {
            let function = BuiltInFunction::from_syntax_node(source, function_node)?;

            FunctionCall::BuiltIn(Box::new(function))
        } else {
            let identifier = Identifier::from_syntax_node(source, function_node)?;

            FunctionCall::ContextDefined {
                name: identifier,
                arguments,
            }
        };

        Ok(Yield { call })
    }

    fn run(&self, source: &str, context: &mut crate::Map) -> Result<Value> {
        self.call.run(source, context)
    }
}
