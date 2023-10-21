use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Expression, Result, Value, VariableMap};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Tool {
    Output(Vec<Expression>),
}

impl AbstractTree for Tool {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let mut expressions = Vec::new();

        for index in 2..node.child_count() - 1 {
            let expression_node = node.child(index).unwrap();
            let expression = Expression::from_syntax_node(source, expression_node)?;

            expressions.push(expression);
        }

        let tool_node = node.child(1).unwrap();
        let tool = match tool_node.kind() {
            "output" => Tool::Output(expressions),
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "output",
                    actual: tool_node.kind(),
                    location: tool_node.start_position(),
                    relevant_source: source[tool_node.byte_range()].to_string(),
                })
            }
        };

        Ok(tool)
    }

    fn run(&self, source: &str, context: &mut VariableMap) -> Result<Value> {
        match self {
            Tool::Output(expressions) => {
                for expression in expressions {
                    let value = expression.run(source, context)?;

                    println!("{value}");
                }

                Ok(Value::Empty)
            }
        }
    }
}
