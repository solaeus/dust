use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Block, Expression, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct While {
    expression: WithPosition<Expression>,
    block: Block,
}

impl While {
    pub fn new(expression: WithPosition<Expression>, block: Block) -> Self {
        Self { expression, block }
    }
}

impl AbstractTree for While {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        self.expression.node.validate(_context)?;
        self.block.validate(_context)
    }

    fn run(self, context: &Context) -> Result<Action, RuntimeError> {
        while self
            .expression
            .node
            .clone()
            .run(context)?
            .as_return_value()?
            .as_boolean()?
        {
            if let Action::Break = self.block.clone().run(context)? {
                break;
            }
        }

        Ok(Action::None)
    }
}
