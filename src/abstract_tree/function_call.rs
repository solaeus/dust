use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Map, Result, TypeDefinition, Value, BUILT_IN_FUNCTIONS};

use super::expression::Expression;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionCall {
    function: Expression,
    arguments: Vec<Expression>,
}

impl FunctionCall {
    pub fn new(function: Expression, arguments: Vec<Expression>) -> Self {
        Self {
            function,
            arguments,
        }
    }
}

impl AbstractTree for FunctionCall {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        debug_assert_eq!("function_call", node.kind());

        let expression_node = node.child(1).unwrap();
        let function = Expression::from_syntax_node(source, expression_node, context)?;

        let mut arguments = Vec::new();

        for index in 2..node.child_count() - 1 {
            let child = node.child(index).unwrap();

            if child.is_named() {
                let expression = Expression::from_syntax_node(source, child, context)?;

                arguments.push(expression);
            }
        }

        Ok(FunctionCall {
            function,
            arguments,
        })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let function = if let Expression::Identifier(identifier) = &self.function {
            let key = identifier.inner();

            for built_in_function in BUILT_IN_FUNCTIONS {
                if key == built_in_function.name() {
                    let mut arguments = Vec::with_capacity(self.arguments.len());

                    for expression in &self.arguments {
                        let value = expression.run(source, context)?;

                        arguments.push(value);
                    }

                    return built_in_function.run(&arguments);
                }
            }

            if let Some(value) = context.variables()?.get(key) {
                value.as_function().cloned()
            } else {
                return Err(Error::FunctionIdentifierNotFound(identifier.clone()));
            }
        } else {
            let expression_run = self.function.run(source, context)?;

            expression_run.as_function().cloned()
        }?;

        let mut function_context = Map::clone_from(context)?;
        let parameter_expression_pairs = function.parameters().iter().zip(self.arguments.iter());

        for (identifier, expression) in parameter_expression_pairs {
            let key = identifier.clone().take_inner();
            let value = expression.run(source, context)?;

            function_context.variables_mut()?.insert(key, value);
        }

        function.run(source, &mut function_context)
    }

    fn expected_type(&self, context: &Map) -> Result<TypeDefinition> {
        self.function.expected_type(context)
    }
}
