use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Expression, Positioned, Type};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionCall {
    function: Box<Positioned<Expression>>,
    arguments: Vec<Positioned<Expression>>,
}

impl FunctionCall {
    pub fn new(function: Positioned<Expression>, arguments: Vec<Positioned<Expression>>) -> Self {
        FunctionCall {
            function: Box::new(function),
            arguments,
        }
    }
}

impl AbstractTree for FunctionCall {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        if let Type::Function { return_type, .. } = self.function.node.expected_type(_context)? {
            Ok(*return_type)
        } else {
            Err(ValidationError::ExpectedFunction)
        }
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        if let Type::Function { .. } = self.function.node.expected_type(_context)? {
            Ok(())
        } else {
            Err(ValidationError::ExpectedFunction)
        }
    }

    fn run(self, context: &Context) -> Result<Action, RuntimeError> {
        let value = self.function.node.run(context)?.as_return_value()?;
        let function = value.as_function()?;
        let mut arguments = Vec::with_capacity(self.arguments.len());

        for expression in self.arguments {
            let value = expression.node.run(context)?.as_return_value()?;

            arguments.push(value);
        }

        let function_context = Context::inherit_data_from(context)?;

        function.clone().call(arguments, function_context)
    }
}
