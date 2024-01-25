use serde::{Deserialize, Serialize};

use crate::{
    AbstractTree, Error, Expression, Format, FunctionExpression, Map, Result, SyntaxNode,
    SyntaxPosition, Type, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionCall {
    function_expression: FunctionExpression,
    arguments: Vec<Expression>,
    syntax_position: SyntaxPosition,
}

impl FunctionCall {
    pub fn new(
        function_expression: FunctionExpression,
        arguments: Vec<Expression>,
        syntax_position: SyntaxPosition,
    ) -> Self {
        Self {
            function_expression,
            arguments,
            syntax_position,
        }
    }
}

impl AbstractTree for FunctionCall {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "function_call", node)?;

        let function_node = node.child(0).unwrap();
        let function_expression = FunctionExpression::from_syntax(function_node, source, context)?;

        let mut arguments = Vec::new();

        for index in 2..node.child_count() - 1 {
            let child = node.child(index).unwrap();

            if child.is_named() {
                let expression = Expression::from_syntax(child, source, context)?;

                arguments.push(expression);
            }
        }

        Ok(FunctionCall {
            function_expression,
            arguments,
            syntax_position: node.range().into(),
        })
    }

    fn check_type(&self, source: &str, context: &Map) -> Result<()> {
        let function_expression_type = self.function_expression.expected_type(context)?;

        let parameter_types = match function_expression_type {
            Type::Function {
                parameter_types, ..
            } => parameter_types,
            Type::Any => return Ok(()),
            _ => {
                return Err(Error::TypeCheckExpectedFunction {
                    actual: function_expression_type,
                }
                .at_source_position(source, self.syntax_position))
            }
        };

        let required_argument_count =
            parameter_types.iter().fold(
                0,
                |acc, r#type| {
                    if r#type.is_option() {
                        acc
                    } else {
                        acc + 1
                    }
                },
            );

        if self.arguments.len() < required_argument_count {
            return Err(Error::ExpectedFunctionArgumentMinimum {
                minumum: required_argument_count,
                actual: self.arguments.len(),
            });
        }

        for (index, expression) in self.arguments.iter().enumerate() {
            if let Some(r#type) = parameter_types.get(index) {
                let expected_type = expression.expected_type(context)?;

                if let Type::Option(optional_type) = r#type {
                    optional_type.check(&expected_type)?;
                } else {
                    r#type
                        .check(&expression.expected_type(context)?)
                        .map_err(|error| error.at_source_position(source, self.syntax_position))?;
                }
            }
        }

        Ok(())
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let value = match &self.function_expression {
            FunctionExpression::Identifier(identifier) => {
                let key = identifier.inner();
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
            FunctionExpression::Yield(r#yield) => r#yield.run(source, context)?,
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

impl Format for FunctionCall {
    fn format(&self, output: &mut String, indent_level: u8) {
        self.function_expression.format(output, indent_level);
        output.push('(');

        for expression in &self.arguments {
            expression.format(output, indent_level);
        }

        output.push(')');
    }
}
