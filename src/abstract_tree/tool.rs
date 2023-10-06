use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Expression, Result, Value, VariableMap};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum ToolCall {
    Output(Expression),
}

impl AbstractTree for ToolCall {
    fn from_syntax_node(node: Node, source: &str) -> Result<Self> {
        let tool_node = node.child(1).unwrap();
        let tool_name = tool_node.kind();

        match tool_name {
            "output" => {
                let expression_node = node.child(1).unwrap();
                let expression = Expression::from_syntax_node(expression_node, source)?;

                Ok(ToolCall::Output(expression))
            }
            _ => Err(Error::UnexpectedSyntax {
                expected: "output",
                actual: tool_name,
                location: tool_node.start_position(),
                relevant_source: tool_node.kind().to_string(),
            }),
        }
    }

    fn run(&self, context: &mut VariableMap) -> Result<crate::Value> {
        match self {
            ToolCall::Output(expression) => {
                let value = expression.run(context)?;

                println!("{value}")
            }
        }

        Ok(Value::Empty)
    }
}
