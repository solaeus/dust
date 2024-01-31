use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Error, Expression, Format, FunctionExpression, Map, SyntaxNode, SyntaxPosition,
    Type, Value,
};

/// A function being invoked and the arguments it is being passed.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionCall {
    function_expression: FunctionExpression,
    arguments: Vec<Expression>,
    syntax_position: SyntaxPosition,
}

impl FunctionCall {
    /// Returns a new FunctionCall.
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
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self, SyntaxError> {
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

    fn expected_type(&self, context: &Map) -> Result<Type, ValidationError> {
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

    fn check_type(&self, source: &str, context: &Map) -> Result<(), ValidationError> {
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

        if self.arguments.len() != parameter_types.len() {
            return Err(Error::ExpectedFunctionArgumentAmount {
                expected: parameter_types.len(),
                actual: self.arguments.len(),
            }
            .at_source_position(source, self.syntax_position));
        }

        for (index, expression) in self.arguments.iter().enumerate() {
            if let Some(r#type) = parameter_types.get(index) {
                r#type
                    .check(&expression.expected_type(context)?)
                    .map_err(|error| error.at_source_position(source, self.syntax_position))?;
            }
        }

        Ok(())
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value, RuntimeError> {
        let value = match &self.function_expression {
            FunctionExpression::Identifier(identifier) => {
                let key = identifier.inner();
                let variables = context.variables()?;

                if let Some((value, _)) = variables.get(key) {
                    value.clone()
                } else {
                    return Err(Error::VariableIdentifierNotFound(
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
