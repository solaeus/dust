use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
};

use super::{
    expression, AbstractNode, Evaluation, Expression, Type, TypeConstructor, ValueNode,
    WithPosition,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    function_expression: Box<Expression>,
    type_arguments: Option<Vec<TypeConstructor>>,
    value_arguments: Option<Vec<Expression>>,

    #[serde(skip)]
    context: Context,
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
            context: Context::new(None),
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

        self.context.set_parent(context.clone())?;

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
            value_parameters,
            return_type: _,
        } = function_node_type
        {
            if let (Some(parameters), Some(arguments)) = (type_parameters, &self.type_arguments) {
                if parameters.len() != arguments.len() {
                    return Err(ValidationError::WrongTypeArguments {
                        arguments: arguments.clone(),
                        parameters: parameters.clone(),
                    });
                }

                for (identifier, constructor) in parameters.into_iter().zip(arguments.into_iter()) {
                    let r#type = constructor.construct(context)?;

                    self.context.set_type(identifier, r#type)?;
                }
            }

            match (value_parameters, &self.value_arguments) {
                (Some(parameters), Some(arguments)) => {
                    for ((identifier, _), expression) in
                        parameters.iter().zip(arguments.into_iter())
                    {
                        let r#type = if let Some(r#type) = expression.expected_type(context)? {
                            r#type
                        } else {
                            return Err(ValidationError::ExpectedExpression(expression.position()));
                        };

                        self.context.set_type(identifier.clone(), r#type)?;
                    }

                    if parameters.len() != arguments.len() {
                        return Err(ValidationError::WrongValueArguments {
                            parameters,
                            arguments: arguments.clone(),
                        });
                    }
                }
                (Some(parameters), None) => {
                    return Err(ValidationError::WrongValueArguments {
                        parameters,
                        arguments: Vec::with_capacity(0),
                    });
                }
                (None, Some(arguments)) => {
                    return Err(ValidationError::WrongValueArguments {
                        parameters: Vec::with_capacity(0),
                        arguments: arguments.clone(),
                    });
                }
                (None, None) => {}
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
        if let Expression::Value(WithPosition {
            node: ValueNode::BuiltInFunction(function),
            ..
        }) = *self.function_expression
        {
            return function
                .call(context, manage_memory)
                .map(|value_option| value_option.map(|value| Evaluation::Return(value)));
        }

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

        if let (Some(type_parameters), Some(type_arguments)) =
            (function.type_parameters(), self.type_arguments)
        {
            for (identifier, constructor) in
                type_parameters.into_iter().zip(type_arguments.into_iter())
            {
                let r#type = constructor.construct(context)?;

                self.context.set_type(identifier.clone(), r#type)?;
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

                self.context.set_value(identifier.clone(), value)?;
            }
        }

        function.call(&self.context, manage_memory)
    }

    fn expected_type(&self, context: &Context) -> Result<Option<Type>, ValidationError> {
        let expression_type = self.function_expression.expected_type(context)?.ok_or(
            ValidationError::ExpectedExpression(self.function_expression.position()),
        )?;

        let (type_parameters, value_parameters, return_type) = if let Type::Function {
            type_parameters,
            value_parameters,
            return_type,
        } = expression_type
        {
            (type_parameters, value_parameters, return_type)
        } else {
            return Err(ValidationError::ExpectedFunction {
                actual: expression_type,
                position: self.function_expression.position(),
            });
        };

        if let Some(Type::Generic {
            identifier: return_identifier,
            concrete_type: None,
        }) = return_type.clone().map(|r#box| *r#box)
        {
            if let (Some(parameters), Some(arguments)) = (type_parameters, &self.type_arguments) {
                for (identifier, constructor) in parameters.into_iter().zip(arguments.into_iter()) {
                    if identifier == return_identifier {
                        let r#type = constructor.construct(context)?;

                        return Ok(Some(Type::Generic {
                            identifier,
                            concrete_type: Some(Box::new(r#type)),
                        }));
                    }
                }
            }
        }

        Ok(return_type.map(|r#box| *r#box))
    }
}

impl Eq for FunctionCall {}

impl PartialEq for FunctionCall {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl PartialOrd for FunctionCall {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FunctionCall {
    fn cmp(&self, other: &Self) -> Ordering {
        todo!()
    }
}
