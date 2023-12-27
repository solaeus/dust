use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    AbstractTree, Error, Expression, Map, Result, Type, Value, ValueNode, BUILT_IN_FUNCTIONS,
};

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
                    source: source[expression_node.byte_range()].to_string(),
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

                if let Some((value, _)) = variables.get(key) {
                    value.clone()
                } else {
                    return Err(Error::FunctionIdentifierNotFound(
                        identifier.inner().clone(),
                    ));
                }
            }
            Expression::Index(index) => index.run(source, context)?,
            Expression::Math(math) => math.run(source, context)?,
            Expression::Logic(logic) => logic.run(source, context)?,
            Expression::FunctionCall(function_call) => function_call.run(source, context)?,
            Expression::Yield(r#yield) => r#yield.run(source, context)?,
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
            Expression::Value(value_node) => {
                if let ValueNode::Function(function) = value_node {
                    let return_type = function.return_type()?.clone();

                    Ok(return_type)
                } else {
                    value_node.expected_type(context)
                }
            }
            Expression::Identifier(identifier) => {
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
                foobar = (fn message <str>) <str> { message }
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
                foobar = (fn cb <() -> str>) <str> {
                    (cb)
                }

                (foobar (fn) <str> { 'Hiya' })
                ",
            ),
            Ok(Value::String("Hiya".to_string()))
        );
    }

    #[test]
    fn evaluate_built_in_function_call() {
        assert_eq!(evaluate("(output 'Hiya')"), Ok(Value::Option(None)));
    }
}
