use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
};

use super::{AbstractNode, Evaluation, Expression, Type, TypeConstructor};

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
}

impl AbstractNode for FunctionCall {
    fn define_and_validate(
        &self,
        outer_context: &Context,
        manage_memory: bool,
    ) -> Result<(), ValidationError> {
        self.context.set_parent(outer_context.clone())?;
        self.function_expression
            .define_and_validate(outer_context, manage_memory)?;

        if let Some(value_arguments) = &self.value_arguments {
            for expression in value_arguments {
                expression.define_and_validate(outer_context, manage_memory)?;
            }
        }

        let function_node_type =
            if let Some(r#type) = self.function_expression.expected_type(outer_context)? {
                r#type
            } else {
                return Err(ValidationError::ExpectedValueStatement(
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
                    let r#type = constructor.construct(outer_context)?;

                    self.context.set_type(identifier, r#type)?;
                }
            }

            match (value_parameters, &self.value_arguments) {
                (Some(parameters), Some(arguments)) => {
                    for ((identifier, _), expression) in
                        parameters.iter().zip(arguments.into_iter())
                    {
                        let r#type =
                            if let Some(r#type) = expression.expected_type(outer_context)? {
                                r#type
                            } else {
                                return Err(ValidationError::ExpectedValueStatement(
                                    expression.position(),
                                ));
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

            return Ok(());
        }

        Err(ValidationError::ExpectedFunction {
            actual: function_node_type,
            position: self.function_expression.position(),
        })
    }

    fn evaluate(
        self,
        outer_context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let function_position = self.function_expression.position();
        let evaluation = self
            .function_expression
            .evaluate(outer_context, manage_memory)?;
        let value = if let Some(Evaluation::Return(value)) = evaluation {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::ExpectedValueStatement(function_position),
            ));
        };

        if let ValueInner::Function(function) = value.inner().as_ref() {
            if let (Some(type_parameters), Some(type_arguments)) =
                (function.type_parameters(), self.type_arguments)
            {
                for (identifier, constructor) in
                    type_parameters.into_iter().zip(type_arguments.into_iter())
                {
                    let r#type = constructor.construct(outer_context)?;

                    self.context.set_type(identifier.clone(), r#type)?;
                }
            }

            if let (Some(parameters), Some(arguments)) =
                (function.value_parameters(), self.value_arguments)
            {
                for ((identifier, _), expression) in
                    parameters.into_iter().zip(arguments.into_iter())
                {
                    let position = expression.position();
                    let evaluation = expression.evaluate(outer_context, manage_memory)?;
                    let value = if let Some(Evaluation::Return(value)) = evaluation {
                        value
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedValueStatement(position),
                        ));
                    };

                    self.context.set_value(identifier.clone(), value)?;
                }
            }

            return function.clone().call(&self.context, manage_memory);
        }

        if let ValueInner::BuiltInFunction(function) = value.inner().as_ref() {
            let (type_parameters, value_parameters, _) = if let Type::Function {
                type_parameters,
                value_parameters,
                return_type,
            } = function.r#type()
            {
                (type_parameters, value_parameters, return_type)
            } else {
                return Err(RuntimeError::ValidationFailure(
                    ValidationError::ExpectedFunction {
                        actual: function.r#type(),
                        position: function_position,
                    },
                ));
            };

            if let (Some(type_parameters), Some(type_arguments)) =
                (type_parameters, self.type_arguments)
            {
                for (identifier, constructor) in
                    type_parameters.into_iter().zip(type_arguments.into_iter())
                {
                    let r#type = constructor.construct(outer_context)?;

                    self.context.set_type(identifier.clone(), r#type)?;
                }
            }

            if let (Some(parameters), Some(arguments)) = (value_parameters, self.value_arguments) {
                for ((identifier, _), expression) in
                    parameters.into_iter().zip(arguments.into_iter())
                {
                    let position = expression.position();
                    let evaluation = expression.evaluate(outer_context, manage_memory)?;
                    let value = if let Some(Evaluation::Return(value)) = evaluation {
                        value
                    } else {
                        return Err(RuntimeError::ValidationFailure(
                            ValidationError::ExpectedValueStatement(position),
                        ));
                    };

                    self.context.set_value(identifier.clone(), value)?;
                }
            }

            return function
                .call(&self.context, manage_memory)
                .map(|option| option.map(|value| Evaluation::Return(value)));
        }

        Err(RuntimeError::ValidationFailure(
            ValidationError::ExpectedFunction {
                actual: value.r#type(outer_context)?,
                position: function_position,
            },
        ))
    }

    fn expected_type(&self, context: &Context) -> Result<Option<Type>, ValidationError> {
        let expression_type = self.function_expression.expected_type(context)?.ok_or(
            ValidationError::ExpectedValueStatement(self.function_expression.position()),
        )?;

        let (type_parameters, return_type) = if let Type::Function {
            type_parameters,
            return_type,
            ..
        } = expression_type
        {
            (type_parameters, return_type)
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

impl Display for FunctionCall {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let FunctionCall {
            function_expression,
            type_arguments,
            value_arguments,
            ..
        } = self;

        write!(f, "{function_expression}")?;

        if let Some(type_arguments) = type_arguments {
            write!(f, "::<")?;

            for constructor in type_arguments {
                write!(f, "{constructor}, ")?;
            }

            write!(f, ">")?;
        }

        write!(f, "(")?;

        if let Some(value_arguments) = value_arguments {
            for expression in value_arguments {
                write!(f, "{expression}, ")?;
            }
        }

        write!(f, ")")?;

        Ok(())
    }
}

impl Eq for FunctionCall {}

impl PartialEq for FunctionCall {
    fn eq(&self, other: &Self) -> bool {
        self.function_expression == other.function_expression
            && self.type_arguments == other.type_arguments
            && self.value_arguments == other.value_arguments
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
