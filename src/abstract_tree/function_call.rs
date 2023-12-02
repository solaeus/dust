use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    AbstractTree, Error, Identifier, Map, Result, TypeDefinition, Value, BUILT_IN_FUNCTIONS,
};

use super::expression::Expression;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionCall {
    function_name: Identifier,
    arguments: Vec<Expression>,
}

impl FunctionCall {
    pub fn new(function_name: Identifier, arguments: Vec<Expression>) -> Self {
        Self {
            function_name,
            arguments,
        }
    }
}

impl AbstractTree for FunctionCall {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        debug_assert_eq!("function_call", node.kind());

        let identifier_node = node.child(1).unwrap();
        let function_name = Identifier::from_syntax_node(source, identifier_node, context)?;

        let mut arguments = Vec::new();

        for index in 2..node.child_count() - 1 {
            let child = node.child(index).unwrap();

            if child.is_named() {
                let expression = Expression::from_syntax_node(source, child, context)?;

                arguments.push(expression);
            }
        }

        let function_type = function_name.expected_type(context)?;
        let function_call = FunctionCall {
            function_name,
            arguments,
        };

        function_type.abstract_check(
            &function_call.expected_type(context)?,
            context,
            node,
            source,
        )?;

        Ok(function_call)
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let key = self.function_name.inner();

        for built_in_function in BUILT_IN_FUNCTIONS {
            if key == built_in_function.name() {
                let mut arguments = Vec::with_capacity(self.arguments.len());

                for expression in &self.arguments {
                    let value = expression.run(source, context)?;

                    arguments.push(value);
                }

                return built_in_function.run(&arguments, context);
            }
        }

        let variables = context.variables()?;
        let function = if let Some(value) = variables.get(key) {
            value.as_function()?
        } else {
            return Err(Error::FunctionIdentifierNotFound(
                self.function_name.clone(),
            ));
        };

        function.call(&self.arguments, source, context)
    }

    fn expected_type(&self, context: &Map) -> Result<TypeDefinition> {
        let function_name = self.function_name.inner();

        if let Some(value) = context.variables()?.get(function_name) {
            let return_type = value.as_function()?.return_type();

            Ok(return_type.clone())
        } else {
            self.function_name.expected_type(context)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{evaluate, Value};

    #[test]
    fn evaluate_function_call() {
        assert_eq!(
            evaluate(
                "
                foobar <fn str -> str> |message| { message }
                (foobar 'Hiya')
                ",
            ),
            Ok(Value::String("Hiya".to_string()))
        );
    }

    #[test]
    fn evaluate_built_in_function_call() {
        assert_eq!(evaluate("(output 'Hiya')"), Ok(Value::Empty));
    }
}
