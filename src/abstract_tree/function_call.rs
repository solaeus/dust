use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    AbstractTree, Error, Map, Result, Type, TypeDefinition, Value, ValueNode, BUILT_IN_FUNCTIONS,
};

use super::expression::Expression;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionCall {
    function_expression: Expression,
    arguments: Vec<Expression>,
}

impl FunctionCall {
    pub fn new(function_expression: Expression, arguments: Vec<Expression>) -> Self {
        Self {
            function_expression,
            arguments,
        }
    }
}

impl AbstractTree for FunctionCall {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        debug_assert_eq!("function_call", node.kind());

        let expression_node = node.child(1).unwrap();
        let function_expression = Expression::from_syntax_node(source, expression_node, context)?;

        let mut arguments = Vec::new();

        for index in 2..node.child_count() - 1 {
            let child = node.child(index).unwrap();

            if child.is_named() {
                let expression = Expression::from_syntax_node(source, child, context)?;

                arguments.push(expression);
            }
        }

        let function_type = function_expression.expected_type(context)?.take_inner();

        if let Type::Function {
            parameter_types,
            return_type,
        } = function_type
        {
            let argument_type_pairs = arguments.iter().zip(parameter_types.iter());

            for (argument, r#type) in argument_type_pairs {
                let argument_type = argument.expected_type(context)?;

                r#type.check(argument_type.inner(), context, node, source)?;
            }
        }

        let function_call = FunctionCall {
            function_expression,
            arguments,
        };

        Ok(function_call)
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let value = match &self.function_expression {
            Expression::Value(value_node) => value_node.run(source, context)?,
            Expression::Identifier(identifier) => {
                let key = identifier.inner();

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
                if let Some(value) = variables.get(key) {
                    value.clone()
                } else {
                    return Err(Error::FunctionIdentifierNotFound(identifier.clone()));
                }
            }
            Expression::Index(index) => index.run(source, context)?,
            Expression::Math(math) => math.run(source, context)?,
            Expression::Logic(logic) => logic.run(source, context)?,
            Expression::FunctionCall(function_call) => function_call.run(source, context)?,
            Expression::Yield(r#yield) => r#yield.run(source, context)?,
        };

        value.as_function()?.call(&self.arguments, source, context)
    }

    fn expected_type(&self, context: &Map) -> Result<TypeDefinition> {
        match &self.function_expression {
            Expression::Value(value_node) => {
                if let ValueNode::Function(function) = value_node {
                    let return_type = function.return_type()?.clone();

                    Ok(TypeDefinition::new(return_type))
                } else {
                    value_node.expected_type(context)
                }
            }
            Expression::Identifier(identifier) => {
                let function_name = identifier.inner();

                if let Some(value) = context.variables()?.get(function_name) {
                    let return_type = value.as_function()?.return_type()?.clone();

                    Ok(TypeDefinition::new(return_type))
                } else {
                    self.function_expression.expected_type(context)
                }
            }
            Expression::Index(index) => index.expected_type(context),
            Expression::Math(math) => math.expected_type(context),
            Expression::Logic(logic) => logic.expected_type(context),
            Expression::FunctionCall(function_call) => function_call.expected_type(context),
            Expression::Yield(r#yield) => r#yield.expected_type(context),
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
                foobar <(str) -> str> = fn |message| { message }
                (foobar 'Hiya')
                ",
            ),
            Ok(Value::String("Hiya".to_string()))
        );
    }

    #[test]
    fn evaluate_callback() {
        assert_eq!(
            evaluate(
                "
                foobar <(() -> str) -> str> = fn |cb| {
                    (cb)
                }

                (foobar fn || { 'Hiya' })
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
