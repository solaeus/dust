use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
};

use super::{AbstractTree, Action, Expression, Type, WithPosition};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionCall {
    function: Box<WithPosition<Expression>>,
    arguments: Vec<WithPosition<Expression>>,
}

impl FunctionCall {
    pub fn new(
        function: WithPosition<Expression>,
        arguments: Vec<WithPosition<Expression>>,
    ) -> Self {
        FunctionCall {
            function: Box::new(function),
            arguments,
        }
    }
}

impl AbstractTree for FunctionCall {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        let function_node_type = self.function.node.expected_type(_context)?;

        if let Type::Function { return_type, .. } = function_node_type {
            Ok(*return_type)
        } else {
            Err(ValidationError::ExpectedFunction {
                actual: function_node_type,
                position: self.function.position,
            })
        }
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        let function_node_type = self.function.node.expected_type(_context)?;

        if let Type::Function { .. } = function_node_type {
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

        function_context.inherit_data_from(&context)?;
        function.clone().call(arguments, function_context)
    }
}
