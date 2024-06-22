use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
};

use super::{AbstractNode, Evaluation, Expression, Type, TypeConstructor};

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

    pub fn function(&self) -> &Box<Expression> {
        &self.function
    }
}

impl AbstractNode for FunctionCall {
    fn define_types(&self, _context: &Context) -> Result<(), ValidationError> {
        self.function.define_types(_context)?;

        let mut previous = ();

        for expression in &self.value_arguments {
            previous = expression.define_types(_context)?;
        }

        Ok(previous)
    }

    fn validate(&self, context: &Context, manage_memory: bool) -> Result<(), ValidationError> {
        self.function.validate(context, manage_memory)?;

        for expression in &self.value_arguments {
            expression.validate(context, manage_memory)?;
        }

        let function_node_type = if let Some(r#type) = self.function.expected_type(context)? {
            r#type
        } else {
            return Err(ValidationError::ExpectedExpression(
                self.function.position(),
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
                position: self.function.position(),
            })
        }
    }

    fn evaluate(
        self,
        context: &Context,
        clear_variables: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let function_position = self.function.position();
        let evaluation = self.function.evaluate(context, clear_variables)?;
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
        let mut arguments = Vec::with_capacity(self.value_arguments.len());

        for expression in self.value_arguments {
            let expression_position = expression.position();
            let action = expression.evaluate(context, clear_variables)?;
            let evalution = if let Some(Evaluation::Return(value)) = action {
                value
            } else {
                return Err(RuntimeError::ValidationFailure(
                    ValidationError::ExpectedExpression(expression_position),
                ));
            };

            arguments.push(evalution);
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

    fn expected_type(&self, context: &Context) -> Result<Option<Type>, ValidationError> {
        let function_type = if let Some(r#type) = self.function.expected_type(context)? {
            r#type
        } else {
            return Err(ValidationError::ExpectedExpression(
                self.function.position(),
            ));
        };

        if let Type::Function {
            return_type,
            type_parameters,
            ..
        } = function_type
        {
            let return_type = return_type.map(|r#box| *r#box);

            if let Some(Type::Generic {
                identifier: return_identifier,
                ..
            }) = &return_type
            {
                if let (Some(type_arguments), Some(type_parameters)) =
                    (&self.type_arguments, &type_parameters)
                {
                    for (constructor, identifier) in
                        type_arguments.into_iter().zip(type_parameters.into_iter())
                    {
                        if identifier == return_identifier {
                            let concrete_type = constructor.clone().construct(&context)?;

                            return Ok(Some(Type::Generic {
                                identifier: identifier.clone(),
                                concrete_type: Some(Box::new(concrete_type)),
                            }));
                        }
                    }
                }

                if let (None, Some(type_parameters)) = (&self.type_arguments, type_parameters) {
                    for (expression, identifier) in (&self.value_arguments)
                        .into_iter()
                        .zip(type_parameters.into_iter())
                    {
                        if &identifier == return_identifier {
                            let concrete_type =
                                if let Some(r#type) = expression.expected_type(context)? {
                                    r#type
                                } else {
                                    return Err(ValidationError::ExpectedExpression(
                                        expression.position(),
                                    ));
                                };

                            return Ok(Some(Type::Generic {
                                identifier,
                                concrete_type: Some(Box::new(concrete_type)),
                            }));
                        }
                    }
                }
            }

            Ok(return_type)
        } else {
            Err(ValidationError::ExpectedFunction {
                actual: function_type,
                position: self.function.position(),
            })
        }
    }
}
