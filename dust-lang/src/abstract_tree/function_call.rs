use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
};

use super::{AbstractNode, Action, Expression, Type, WithPosition};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FunctionCall {
    function: Box<Expression>,
    type_arguments: Vec<WithPosition<Type>>,
    arguments: Vec<Expression>,
}

impl FunctionCall {
    pub fn new(
        function: Expression,
        type_arguments: Vec<WithPosition<Type>>,
        arguments: Vec<Expression>,
    ) -> Self {
        FunctionCall {
            function: Box::new(function),
            type_arguments,
            arguments,
        }
    }
}

impl AbstractNode for FunctionCall {
    fn expected_type(&self, _context: &mut Context) -> Result<Type, ValidationError> {
        let function_node_type = self.function.expected_type(_context)?;

        if let Type::Function { return_type, .. } = function_node_type {
            Ok(return_type.item)
        } else {
            Err(ValidationError::ExpectedFunction {
                actual: function_node_type,
                position: self.function.position(),
            })
        }
    }

    fn validate(&self, context: &mut Context, manage_memory: bool) -> Result<(), ValidationError> {
        self.function.validate(context, manage_memory)?;

        for expression in &self.arguments {
            expression.validate(context, manage_memory)?;
        }

        let function_node_type = self.function.expected_type(context)?;

        if let Type::Function {
            parameter_types,
            return_type: _,
        } = function_node_type
        {
            for (type_parameter, type_argument) in
                parameter_types.iter().zip(self.type_arguments.iter())
            {
                if let Type::Argument(_) = type_parameter.item {
                    continue;
                }

                type_parameter
                    .item
                    .check(&type_argument.item)
                    .map_err(|conflict| ValidationError::TypeCheck {
                        conflict,
                        actual_position: type_argument.position,
                        expected_position: type_parameter.position,
                    })?;
            }

            for (type_parameter, expression) in parameter_types.iter().zip(self.arguments.iter()) {
                if let Type::Argument(_) = type_parameter.item {
                    continue;
                }

                let actual = expression.expected_type(context)?;

                type_parameter.item.check(&actual).map_err(|conflict| {
                    ValidationError::TypeCheck {
                        conflict,
                        actual_position: expression.position(),
                        expected_position: type_parameter.position,
                    }
                })?;
            }

            Ok(())
        } else {
            Err(ValidationError::ExpectedFunction {
                actual: function_node_type,
                position: self.function.position(),
            })
        }
    }

    fn run(self, context: &mut Context, clear_variables: bool) -> Result<Action, RuntimeError> {
        let function_position = self.function.position();
        let action = self.function.run(context, clear_variables)?;
        let value = if let Action::Return(value) = action {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::InterpreterExpectedReturn(function_position),
            ));
        };
        let function = if let ValueInner::Function(function) = value.inner().as_ref() {
            function
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::ExpectedFunction {
                    actual: value.r#type(context)?,
                    position: function_position,
                },
            ));
        };
        let mut arguments = Vec::with_capacity(self.arguments.len());

        for expression in self.arguments {
            let expression_position = expression.position();
            let action = expression.run(context, clear_variables)?;
            let value = if let Action::Return(value) = action {
                value
            } else {
                return Err(RuntimeError::ValidationFailure(
                    ValidationError::InterpreterExpectedReturn(expression_position),
                ));
            };

            arguments.push(value);
        }

        let mut function_context = Context::new(Some(&context));

        for (type_parameter, type_argument) in function
            .type_parameters()
            .iter()
            .map(|r#type| r#type.item.clone())
            .zip(self.type_arguments.into_iter().map(|r#type| r#type.item))
        {
            if let Type::Argument(identifier) = type_parameter {
                function_context.set_type(identifier, type_argument)?;
            }
        }

        function
            .clone()
            .call(arguments, &mut function_context, clear_variables)
    }
}
