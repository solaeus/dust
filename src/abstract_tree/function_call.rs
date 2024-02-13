use serde::{Deserialize, Serialize};

use crate::{
    built_in_functions::Callable,
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Context, Expression, Format, Function, FunctionExpression, SourcePosition,
    SyntaxNode, Type, Value,
};

/// A function being invoked and the arguments it is being passed.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionCall {
    function_expression: FunctionExpression,
    arguments: Vec<Expression>,
    syntax_position: SourcePosition,

    #[serde(skip)]
    context: Context,
}

impl FunctionCall {
    /// Returns a new FunctionCall.
    pub fn new(
        function_expression: FunctionExpression,
        arguments: Vec<Expression>,
        syntax_position: SourcePosition,
        context: Context,
    ) -> Self {
        Self {
            function_expression,
            arguments,
            syntax_position,
            context,
        }
    }
}

impl AbstractTree for FunctionCall {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(source, "function_call", node)?;

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
            context: Context::new(),
        })
    }

    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError> {
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
        }
    }

    fn validate(&self, _source: &str, context: &Context) -> Result<(), ValidationError> {
        let function_expression_type = self.function_expression.expected_type(context)?;

        let parameter_types = if let Type::Function {
            parameter_types, ..
        } = function_expression_type
        {
            parameter_types
        } else {
            return Err(ValidationError::TypeCheckExpectedFunction {
                actual: function_expression_type,
                position: self.syntax_position,
            });
        };

        if self.arguments.len() != parameter_types.len() {
            return Err(ValidationError::ExpectedFunctionArgumentAmount {
                expected: parameter_types.len(),
                actual: self.arguments.len(),
                position: self.syntax_position,
            });
        }

        for (index, expression) in self.arguments.iter().enumerate() {
            if let Some(expected) = parameter_types.get(index) {
                let actual = expression.expected_type(context)?;

                if !expected.accepts(&actual) {
                    return Err(ValidationError::TypeCheck {
                        expected: expected.clone(),
                        actual,
                        position: self.syntax_position,
                    });
                }
            }
        }

        Ok(())
    }

    fn run(&self, source: &str, context: &Context) -> Result<Value, RuntimeError> {
        let value = match &self.function_expression {
            FunctionExpression::Identifier(identifier) => {
                let key = identifier.inner();

                if let Some(value) = context.get_value(key)? {
                    self.context
                        .set_value(identifier.inner().clone(), value.clone())?;
                    value.clone()
                } else {
                    return Err(RuntimeError::VariableIdentifierNotFound(
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
        let function = value.as_function()?;

        match function {
            Function::BuiltIn(built_in_function) => {
                let mut arguments = Vec::with_capacity(self.arguments.len());

                for expression in &self.arguments {
                    let value = expression.run(source, context)?;

                    arguments.push(value);
                }

                built_in_function.call(&arguments, source, &self.context)
            }
            Function::ContextDefined(function_node) => {
                let parameter_expression_pairs =
                    function_node.parameters().iter().zip(self.arguments.iter());

                for (identifier, expression) in parameter_expression_pairs {
                    let value = expression.run(source, context)?;

                    self.context.set_value(identifier.inner().clone(), value)?;
                }

                function_node.call(source, &self.context)
            }
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
