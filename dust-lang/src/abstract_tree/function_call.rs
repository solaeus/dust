use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::{Function, ParsedFunction, ValueInner},
};

use super::{AbstractNode, Action, Expression, Type, WithPosition};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionCall {
    function: Box<WithPosition<Expression>>,
    type_arguments: Vec<WithPosition<Type>>,
    arguments: Vec<WithPosition<Expression>>,
}

impl FunctionCall {
    pub fn new(
        function: WithPosition<Expression>,
        type_arguments: Vec<WithPosition<Type>>,
        arguments: Vec<WithPosition<Expression>>,
    ) -> Self {
        FunctionCall {
            function: Box::new(function),
            type_arguments,
            arguments,
        }
    }
}

impl AbstractNode for FunctionCall {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        let function_node_type = self.function.node.expected_type(_context)?;

        if let Type::Function { return_type, .. } = function_node_type {
            Ok(return_type.node)
        } else {
            Err(ValidationError::ExpectedFunction {
                actual: function_node_type,
                position: self.function.position,
            })
        }
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        for expression in &self.arguments {
            expression.node.validate(context)?;
        }

        let function_node_type = self.function.node.expected_type(context)?;

        if let Type::Function {
            parameter_types,
            return_type: _,
        } = function_node_type
        {
            for (type_parameter, type_argument) in
                parameter_types.iter().zip(self.type_arguments.iter())
            {
                type_parameter
                    .node
                    .check(&type_argument.node)
                    .map_err(|conflict| ValidationError::TypeCheck {
                        conflict,
                        actual_position: type_argument.position,
                        expected_position: type_parameter.position,
                    })?;
            }

            for (type_parameter, expression) in parameter_types.iter().zip(self.arguments.iter()) {
                let actual = expression.node.expected_type(context)?;

                type_parameter.node.check(&actual).map_err(|conflict| {
                    ValidationError::TypeCheck {
                        conflict,
                        actual_position: expression.position,
                        expected_position: type_parameter.position,
                    }
                })?;
            }

            Ok(())
        } else {
            Err(ValidationError::ExpectedFunction {
                actual: function_node_type,
                position: self.function.position,
            })
        }
    }

    fn run(self, context: &Context) -> Result<Action, RuntimeError> {
        let action = self.function.node.run(context)?;
        let value = if let Action::Return(value) = action {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::InterpreterExpectedReturn(self.function.position),
            ));
        };
        let function = if let ValueInner::Function(function) = value.inner().as_ref() {
            function
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::ExpectedFunction {
                    actual: value.r#type(context)?,
                    position: self.function.position,
                },
            ));
        };
        let mut arguments = Vec::with_capacity(self.arguments.len());

        for expression in self.arguments {
            let action = expression.node.run(context)?;
            let value = if let Action::Return(value) = action {
                value
            } else {
                return Err(RuntimeError::ValidationFailure(
                    ValidationError::InterpreterExpectedReturn(expression.position),
                ));
            };

            arguments.push(value);
        }

        let function_context = Context::new();

        if let Function::Parsed(ParsedFunction {
            type_parameters, ..
        }) = function
        {
            for (type_parameter, type_argument) in type_parameters
                .iter()
                .map(|r#type| r#type.node.clone())
                .zip(self.type_arguments.into_iter().map(|r#type| r#type.node))
            {
                if let Type::Argument(identifier) = type_parameter {
                    function_context.set_type(identifier, type_argument)?;
                }
            }
        };

        function_context.inherit_data_from(&context)?;
        function.clone().call(arguments, function_context)
    }
}
