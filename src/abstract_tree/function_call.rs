use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Expression, Type};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionCall {
    function: Box<Expression>,
    arguments: Vec<Expression>,
}

impl FunctionCall {
    pub fn new(function: Expression, arguments: Vec<Expression>) -> Self {
        FunctionCall {
            function: Box::new(function),
            arguments,
        }
    }
}

impl AbstractTree for FunctionCall {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        if let Type::Function { return_type, .. } = self.function.expected_type(_context)? {
            Ok(*return_type)
        } else {
            Err(ValidationError::ExpectedFunction)
        }
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        if let Type::Function { .. } = self.function.expected_type(_context)? {
            Ok(())
        } else {
            Err(ValidationError::ExpectedFunction)
        }
    }

    fn run(self, context: &Context) -> Result<Action, RuntimeError> {
        let value = self.function.run(context)?.as_value()?;
        let function = value.as_function()?;
        let mut arguments = Vec::with_capacity(self.arguments.len());

        for expression in self.arguments {
            let value = expression.run(context)?.as_value()?;

            arguments.push(value);
        }

        let function_context = Context::with_data_from(context)?;

        function.call(arguments, function_context)
    }
}
