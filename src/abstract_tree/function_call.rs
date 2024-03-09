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
        todo!()
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        todo!()
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        todo!()
    }
}
