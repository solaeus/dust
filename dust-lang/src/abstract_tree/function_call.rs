use std::cmp::Ordering;

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

    pub fn function_expression(&self) -> &Box<Expression> {
        &self.function_expression
    }
}

impl AbstractNode for FunctionCall {
    fn define_types(&self, context: &Context) -> Result<(), ValidationError> {
        self.function_expression.define_types(context)?;

        let (type_parameters, value_parameters) =
            if let Some(r#type) = self.function_expression.expected_type(context)? {
                if let Type::Function {
                    type_parameters,
                    value_parameters,
                    ..
                } = r#type
                {
                    (type_parameters, value_parameters)
                } else {
                    return Err(ValidationError::ExpectedFunction {
                        actual: r#type,
                        position: self.function_expression.position(),
                    });
                }
            } else {
                todo!("Create an error for this occurence");
            };

        if let (Some(type_parameters), Some(type_arguments)) =
            (type_parameters, &self.type_arguments)
        {
            for (identifier, constructor) in
                type_parameters.into_iter().zip(type_arguments.into_iter())
            {
                let r#type = constructor.construct(context)?;

                self.context.set_type(identifier, r#type)?;
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
        clear_variables: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let function_position = self.function_expression.position();
        let evaluation = self
            .function_expression
            .evaluate(context, clear_variables)?;
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

        match (function.type_parameters(), self.type_arguments) {
            (Some(type_parameters), Some(type_arguments)) => {
                for (parameter, constructor) in
                    type_parameters.into_iter().zip(type_arguments.into_iter())
                {
                    let r#type = constructor.construct(context)?;

                    self.context.set_type(parameter.clone(), r#type)?;
                }
            }
            _ => {}
        }

        if let (Some(value_parameters), Some(value_arguments)) =
            (function.value_parameters(), self.value_arguments)
        {
            for ((identifier, _), expression) in
                value_parameters.into_iter().zip(value_arguments.iter())
            {
                let expression_position = expression.position();
                let evaluation = expression.clone().evaluate(context, clear_variables)?;
                let value = if let Some(Evaluation::Return(value)) = evaluation {
                    value
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedExpression(expression_position),
                    ));
                };

                self.context.set_value(identifier.clone(), value)?;
            }
        }

        function.clone().call(&self.context, clear_variables)
    }

    fn expected_type(&self, context: &Context) -> Result<Option<Type>, ValidationError> {
        let return_type = if let Some(r#type) = self.function_expression.expected_type(context)? {
            if let Type::Function {
                type_parameters,
                value_parameters,
                return_type,
            } = r#type
            {
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
        let function_cmp = self.function_expression.cmp(&other.function_expression);

        if function_cmp.is_eq() {
            let type_arg_cmp = self.type_arguments.cmp(&other.type_arguments);

            if type_arg_cmp.is_eq() {
                self.value_arguments.cmp(&other.value_arguments)
            } else {
                type_arg_cmp
            }
        } else {
            function_cmp
        }
    }
}
