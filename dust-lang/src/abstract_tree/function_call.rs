use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
};

use super::{AbstractNode, Evaluation, Expression, Type, TypeConstructor};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FunctionCall {
    function_expression: Box<Expression>,
    type_arguments: Option<Vec<TypeConstructor>>,
    value_arguments: Option<Vec<Expression>>,
}

impl FunctionCall {
    pub fn new(
        function_expression: Expression,
        type_arguments: Option<Vec<TypeConstructor>>,
        value_arguments: Option<Vec<Expression>>,
    ) -> Self {
        FunctionCall {
            function_expression: Box::new(function_expression),
            type_arguments,
            value_arguments,
        }
    }

    pub fn function_expression(&self) -> &Box<Expression> {
        &self.function_expression
    }
}

impl AbstractNode for FunctionCall {
    fn define_types(&self, context: &Context) -> Result<(), ValidationError> {
        self.function_expression.define_types(context)?;

        if let Some(expressions) = &self.value_arguments {
            for expression in expressions {
                expression.define_types(context)?;
            }
        }

        Ok(())
    }

    fn validate(&self, context: &Context, manage_memory: bool) -> Result<(), ValidationError> {
        self.function_expression.validate(context, manage_memory)?;

        if let Some(value_arguments) = &self.value_arguments {
            for expression in value_arguments {
                expression.validate(context, manage_memory)?;
            }
        }

        let function_node_type =
            if let Some(r#type) = self.function_expression.expected_type(context)? {
                r#type
            } else {
                return Err(ValidationError::ExpectedExpression(
                    self.function_expression.position(),
                ));
            };

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
                position: self.function_expression.position(),
            })
        }
    }

    fn evaluate(
        self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let function_position = self.function_expression.position();
        let evaluation = self.function_expression.evaluate(context, manage_memory)?;
        let value = if let Some(Evaluation::Return(value)) = evaluation {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::ExpectedExpression(function_position),
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

        let function_context = Context::new(None);

        if let Some(type_parameters) = function.type_parameters() {
            for identifier in type_parameters {
                function_context.set_type(
                    identifier.clone(),
                    Type::Generic {
                        identifier: identifier.clone(),
                        concrete_type: None,
                    },
                )?;
            }
        }

        if let (Some(parameters), Some(arguments)) =
            (function.value_parameters(), self.value_arguments)
        {
            for ((identifier, _), expression) in parameters.into_iter().zip(arguments.into_iter()) {
                let position = expression.position();
                let evaluation = expression.evaluate(context, manage_memory)?;
                let value = if let Some(Evaluation::Return(value)) = evaluation {
                    value
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedValue(position),
                    ));
                };

                function_context.set_value(identifier.clone(), value)?;
            }
        }

        function.call(&function_context, manage_memory)
    }

    fn expected_type(&self, context: &Context) -> Result<Option<Type>, ValidationError> {
        let return_type = if let Some(r#type) = self.function_expression.expected_type(context)? {
            if let Type::Function { return_type, .. } = r#type {
                return_type
            } else {
                return Err(ValidationError::ExpectedFunction {
                    actual: r#type,
                    position: self.function_expression.position(),
                });
            }
        } else {
            return Err(ValidationError::ExpectedExpression(
                self.function_expression.position(),
            ));
        };

        let return_type = return_type.map(|r#box| *r#box);

        Ok(return_type)
    }
}
