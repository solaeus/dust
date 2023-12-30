use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    AbstractTree, Error, Expression, FunctionExpression, Map, Result, Type, Value,
    BUILT_IN_FUNCTIONS,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionCall {
    function_expression: FunctionExpression,
    arguments: Vec<Expression>,
}

impl FunctionCall {
    pub fn new(function_expression: FunctionExpression, arguments: Vec<Expression>) -> Self {
        Self {
            function_expression,
            arguments,
        }
    }
}

impl AbstractTree for FunctionCall {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "function_call", node)?;

        let function_node = node.child(0).unwrap();
        let function_expression =
            FunctionExpression::from_syntax_node(source, function_node, context)?;
        let function_type = function_expression.expected_type(context)?;

        let mut minimum_parameter_count = 0;
        let mut arguments = Vec::new();

        for index in 2..node.child_count() - 1 {
            let child = node.child(index).unwrap();

            if child.is_named() {
                let expression = Expression::from_syntax_node(source, child, context)?;
                let expression_type = expression.expected_type(context)?;
                let argument_index = arguments.len();

                if let Type::Function {
                    parameter_types, ..
                } = &function_type
                {
                    if let Some(r#type) = parameter_types.get(argument_index) {
                        if let Type::Option(_) = r#type {
                        } else {
                            minimum_parameter_count += 1;
                        }

                        r#type
                            .check(&expression_type)
                            .map_err(|error| error.at_node(child, source))?;
                    }
                }

                arguments.push(expression);
            }
        }

        if let Type::Function {
            parameter_types: _, ..
        } = &function_type
        {
            if arguments.len() < minimum_parameter_count {
                return Err(Error::ExpectedFunctionArgumentMinimum {
                    source: source[function_node.byte_range()].to_string(),
                    minumum_expected: minimum_parameter_count,
                    actual: arguments.len(),
                });
            }
        }

        Ok(FunctionCall {
            function_expression,
            arguments,
        })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let value = match &self.function_expression {
            FunctionExpression::Identifier(identifier) => {
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

                if let Some((value, _)) = variables.get(key) {
                    value.clone()
                } else {
                    return Err(Error::FunctionIdentifierNotFound(
                        identifier.inner().clone(),
                    ));
                }
            }
            FunctionExpression::FunctionCall(function_call) => {
                function_call.run(source, context)?
            }
            FunctionExpression::Value(value_node) => value_node.run(source, context)?,
            FunctionExpression::Index(index) => index.run(source, context)?,
        };

        let mut arguments = Vec::with_capacity(self.arguments.len());

        for expression in &self.arguments {
            let value = expression.run(source, context)?;

            arguments.push(value);
        }

        value.as_function()?.call(&arguments, source, context)
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        match &self.function_expression {
            FunctionExpression::Identifier(identifier) => {
                for built_in_function in BUILT_IN_FUNCTIONS {
                    if identifier.inner() == built_in_function.name() {
                        if let Type::Function {
                            parameter_types: _,
                            return_type,
                        } = built_in_function.r#type()
                        {
                            return Ok(*return_type);
                        }
                    }
                }

                let identifier_type = identifier.expected_type(context)?;

                if let Type::Function {
                    parameter_types: _,
                    return_type,
                } = &identifier_type
                {
                    Ok(*return_type.clone())
                } else {
                    Ok(identifier_type)
                }
            }
            FunctionExpression::FunctionCall(function_call) => function_call.expected_type(context),
            FunctionExpression::Value(value_node) => value_node.expected_type(context),
            FunctionExpression::Index(index) => index.expected_type(context),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{interpret, Value};

    #[test]
    fn evaluate_function_call() {
        assert_eq!(
            interpret(
                "
                foobar = (message <str>) -> <str> { message }
                foobar('Hiya')
                ",
            ),
            Ok(Value::String("Hiya".to_string()))
        );
    }

    #[test]
    fn evaluate_callback() {
        assert_eq!(
            interpret(
                "
                foobar = (cb <() -> str>) <str> {
                    cb()
                }
                foobar(() <str> { 'Hiya' })
                ",
            ),
            Ok(Value::String("Hiya".to_string()))
        );
    }

    #[test]
    fn evaluate_built_in_function_call() {
        assert_eq!(interpret("output('Hiya')"), Ok(Value::Option(None)));
    }
}
