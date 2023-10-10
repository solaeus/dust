use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{tool::Tool, AbstractTree, Result, Value, VariableMap};

use super::{expression::Expression, identifier::Identifier};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionCall {
    name: FunctionName,
    arguments: Vec<Expression>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum FunctionName {
    Identifier(Identifier),
    Tool(Tool),
}

impl AbstractTree for FunctionCall {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        debug_assert_eq!("function_call", node.kind());

        let name_node = node.child(1).unwrap();

        let name = match name_node.kind() {
            "identifier" => {
                FunctionName::Identifier(Identifier::from_syntax_node(source, name_node)?)
            }
            "tool" => {
                let tool_node = name_node.child(0).unwrap();
                let tool = match tool_node.kind() {
                    "output" => Tool::Output,
                    "read" => Tool::Read,
                    _ => panic!(""),
                };

                FunctionName::Tool(tool)
            }
            _ => panic!(""),
        };

        let mut arguments = Vec::new();

        let mut current_index = 2;
        while current_index < node.child_count() - 1 {
            let expression_node = node.child(current_index).unwrap();
            let expression = Expression::from_syntax_node(source, expression_node)?;

            arguments.push(expression);

            current_index += 1;
        }

        Ok(FunctionCall { name, arguments })
    }

    fn run(&self, source: &str, context: &mut VariableMap) -> Result<Value> {
        let identifier = match &self.name {
            FunctionName::Identifier(identifier) => identifier,
            FunctionName::Tool(tool) => {
                let value = self
                    .arguments
                    .first()
                    .map(|expression| expression.run(source, context))
                    .unwrap_or(Ok(Value::Empty))?;

                return tool.run(&value);
            }
        };
        let key = identifier.inner();
        let definition = if let Some(value) = context.get_value(key)? {
            value.as_function().cloned()?
        } else {
            return Err(crate::Error::FunctionIdentifierNotFound(identifier.clone()));
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
