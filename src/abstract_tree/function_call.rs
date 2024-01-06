use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Expression, FunctionExpression, Map, Result, Type, Value};

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

        let mut arguments = Vec::new();

        for index in 2..node.child_count() - 1 {
            let child = node.child(index).unwrap();

            if child.is_named() {
                let expression = Expression::from_syntax_node(source, child, context)?;

                arguments.push(expression);
            }
        }

        Ok(FunctionCall {
            function_expression,
            arguments,
        })
    }

    fn check_type(&self, _source: &str, context: &Map) -> Result<()> {
        let function_expression_type = self.function_expression.expected_type(context)?;

        let parameter_types = match function_expression_type {
            Type::Function {
                parameter_types, ..
            } => parameter_types,
            Type::Any => return Ok(()),
            _ => {
                return Err(Error::TypeCheckExpectedFunction {
                    actual: function_expression_type,
                })
            }
        };

        for (index, expression) in self.arguments.iter().enumerate() {
            if let Some(r#type) = parameter_types.get(index) {
                r#type.check(&expression.expected_type(context)?)?;
            }
        }

        if self.arguments.len() != parameter_types.len() {
            return Err(Error::ExpectedFunctionArgumentAmount {
                source: "TODO".to_string(),
                expected: parameter_types.len(),
                actual: self.arguments.len(),
            });
        }

        Ok(())
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let (name, value) = match &self.function_expression {
            FunctionExpression::Identifier(identifier) => {
                let key = identifier.inner();
                let variables = context.variables()?;

                if let Some((value, _)) = variables.get(key) {
                    (Some(key.clone()), value.clone())
                } else {
                    return Err(Error::FunctionIdentifierNotFound(
                        identifier.inner().clone(),
                    ));
                }
            }
            FunctionExpression::FunctionCall(function_call) => {
                (None, function_call.run(source, context)?)
            }
            FunctionExpression::Value(value_node) => (None, value_node.run(source, context)?),
            FunctionExpression::Index(index) => (None, index.run(source, context)?),
            FunctionExpression::Yield(r#yield) => (None, r#yield.run(source, context)?),
        };

        let mut arguments = Vec::with_capacity(self.arguments.len());

        for expression in &self.arguments {
            let value = expression.run(source, context)?;

            arguments.push(value);
        }

        if let Some(name) = &name {
            context.set(name.to_string(), value.clone(), None)?;
        }

        value
            .as_function()
            .map_err(|error| {
                println!("{name:?}");

                error
            })?
            .call(name, &arguments, source, context)
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        match &self.function_expression {
            FunctionExpression::Identifier(identifier) => {
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
            FunctionExpression::Value(value_node) => {
                let value_type = value_node.expected_type(context)?;

                if let Type::Function { return_type, .. } = value_type {
                    Ok(*return_type)
                } else {
                    Ok(value_type)
                }
            }
            FunctionExpression::Index(index) => index.expected_type(context),
            FunctionExpression::Yield(r#yield) => r#yield.expected_type(context),
        }
    }
}
