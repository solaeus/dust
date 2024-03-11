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
        Ok(Type::None)
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        self.expression.validate(_context)?;
        self.block.validate(_context)
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        while self
            .expression
            .clone()
            .run(_context)?
            .as_return_value()?
            .as_boolean()?
        {
            if let Action::Break = self.block.clone().run(_context)? {
                break;
            }
        }

        Ok(Action::None)
    }
}
