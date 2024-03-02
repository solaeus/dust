use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    Value,
};

use super::{AbstractTree, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Loop<'src> {
    statements: Vec<Statement<'src>>,
}

impl<'src> Loop<'src> {
    pub fn new(statements: Vec<Statement<'src>>) -> Self {
        Self { statements }
    }
}

impl<'src> AbstractTree for Loop<'src> {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        todo!()
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        todo!()
    }

    fn run(self, _: &Context) -> Result<Value, RuntimeError> {
        todo!()
    }
}
