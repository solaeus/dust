use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
};

use super::{AbstractNode, Evaluation, ExpectedType, Expression, Type, TypeConstructor};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FunctionCall {
    function: Box<Expression>,
    type_arguments: Option<Vec<TypeConstructor>>,
    value_arguments: Vec<Expression>,
}

impl FunctionCall {
    pub fn new(
        function: Expression,
        type_arguments: Option<Vec<TypeConstructor>>,
        value_arguments: Vec<Expression>,
    ) -> Self {
        FunctionCall {
            function: Box::new(function),
            type_arguments,
            value_arguments,
        }
    }
}

impl AbstractNode for FunctionCall {
    fn validate(&self, context: &mut Context, manage_memory: bool) -> Result<(), ValidationError> {
        self.function.validate(context, manage_memory)?;

        for expression in &self.value_arguments {
            expression.validate(context, manage_memory)?;
        }

        let function_node_type = self.function.expected_type(context)?;

        if let Type::Function {
            type_parameters,
            value_parameters: _,
            return_type: _,
        } = function_node_type
        {
            match (type_parameters, &self.type_arguments) {
                (Some(type_parameters), Some(type_arguments)) => {
                    if type_parameters.len() != type_arguments.len() {
                        return Err(ValidationError::WrongTypeArgumentCount {
                            actual: type_parameters.len(),
                            expected: type_arguments.len(),
                        });
                    }
                }
                _ => {}
            }

            Ok(())
        } else {
            Err(ValidationError::ExpectedFunction {
                actual: function_node_type,
                position: self.function.position(),
            })
        }
    }

    fn evaluate(
        self,
        context: &mut Context,
        clear_variables: bool,
    ) -> Result<Evaluation, RuntimeError> {
        let function_position = self.function.position();
        let action = self.function.evaluate(context, clear_variables)?;
        let value = if let Evaluation::Return(value) = action {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::InterpreterExpectedReturn(function_position),
            ));
        };
        let function = if let ValueInner::Function(function) = value.inner().as_ref() {
            function.clone()
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::ExpectedFunction {
                    actual: value.r#type(context)?,
                    position: function_position,
                },
            ));
        };
        let mut arguments = Vec::with_capacity(self.value_arguments.len());

        for expression in self.value_arguments {
            let expression_position = expression.position();
            let action = expression.evaluate(context, clear_variables)?;
            let value = if let Evaluation::Return(value) = action {
                value
            } else {
                return Err(RuntimeError::ValidationFailure(
                    ValidationError::InterpreterExpectedReturn(expression_position),
                ));
            };

            arguments.push(value);
        }

        let mut function_context = Context::new(Some(&context));

        match (function.type_parameters(), self.type_arguments) {
            (Some(type_parameters), Some(type_arguments)) => {
                for (parameter, constructor) in
                    type_parameters.into_iter().zip(type_arguments.into_iter())
                {
                    let r#type = constructor.construct(context)?;

                    function_context.set_type(parameter.clone(), r#type)?;
                }
            }
            _ => {}
        }

        function
            .clone()
            .call(arguments, &mut function_context, clear_variables)
    }
}

impl ExpectedType for FunctionCall {
    fn expected_type(&self, _context: &mut Context) -> Result<Type, ValidationError> {
        let function_node_type = self.function.expected_type(_context)?;

        if let Type::Function { return_type, .. } = function_node_type {
            Ok(*return_type)
        } else {
            Err(ValidationError::ExpectedFunction {
                actual: function_node_type,
                position: self.function.position(),
            })
        }
    }
}
