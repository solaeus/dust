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
                let tool = Tool::new(tool_node.kind())?;

                FunctionName::Tool(tool)
            }
            _ => panic!(""),
        };

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

    fn run(&self, source: &str, context: &mut VariableMap) -> Result<Value> {
        let identifier = match &self.name {
            FunctionName::Identifier(identifier) => identifier,
            FunctionName::Tool(tool) => {
                let mut values = Vec::with_capacity(self.arguments.len());

                for expression in &self.arguments {
                    let value = expression.run(source, context)?;

                    values.push(value);
                }

                return tool.run(&values);
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
