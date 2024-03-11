use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Block, Expression, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct While {
    expression: Expression,
    block: Block,
}

impl While {
    pub fn new(expression: Expression, block: Block) -> Self {
        Self { expression, block }
    }
}

impl AbstractTree for While {
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
